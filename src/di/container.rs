use crate::error::{MeshestraError, Result};
use dashmap::DashMap;
use std::any::{Any, TypeId};
use std::sync::Arc;

/// Thread-safe dependency injection container
///
/// The Container manages service instances and their dependencies.
/// It uses TypeId-based lookup for type-safe service resolution.
pub struct Container {
    /// TypeId → Service instance mapping
    services: DashMap<TypeId, ServiceEntry>,

    /// Trait TypeId → Implementation TypeId mapping
    /// This enables resolving Arc<dyn Trait> requests
    /// Caster function: takes base service (Arc<dyn Any>) and returns Arc<Arc<dyn Trait>> (as Arc<dyn Any>)
    /// Trait TypeId → Implementation TypeId mapping
    trait_mappings: DashMap<TypeId, TypeId>,

    /// Caster function: takes base service (Arc<dyn Any>) and returns Arc<Arc<dyn Trait>> (as Arc<dyn Any>)
    casters: DashMap<TypeId, Box<dyn Fn(Arc<dyn Any + Send + Sync>) -> Arc<dyn Any + Send + Sync> + Send + Sync>>,
}

struct ServiceEntry {
    instance: Arc<dyn Any + Send + Sync>,
    type_name: &'static str,
}

impl Container {
    /// Create a new empty container
    pub fn new() -> Self {
        Self {
            services: DashMap::new(),
            trait_mappings: DashMap::new(),
            casters: DashMap::new(),
        }
    }

    /// Register a service instance in the container
    pub fn register<T: 'static + Send + Sync>(&mut self, instance: T) -> &mut Self {
        let type_id = TypeId::of::<T>();
        let entry = ServiceEntry {
            instance: Arc::new(instance),
            type_name: std::any::type_name::<T>(),
        };

        self.services.insert(type_id, entry);
        self
    }

    /// Register a trait-to-implementation mapping
    pub fn register_trait<Trait, Impl, F>(&mut self, caster_fn: F) -> &mut Self 
    where 
        Trait: ?Sized + 'static + Send + Sync, 
        Impl: 'static + Send + Sync,
        F: Fn(Arc<Impl>) -> Arc<Trait> + 'static + Send + Sync,
    {
        let trait_id = TypeId::of::<Trait>();
        let impl_id = TypeId::of::<Impl>();
        
        self.trait_mappings.insert(trait_id, impl_id);
        
        // Register caster
        let caster = Box::new(move |instance: Arc<dyn Any + Send + Sync>| {
            // 1. Downcast Arc<dyn Any> -> Arc<Impl>
            let concrete = instance.downcast::<Impl>().expect("Failed to downcast to implementation type");
            
            // 2. Coerce Arc<Impl> -> Arc<Trait> using provided generic-erased function
            // (the function is captured)
            let trait_obj = caster_fn(concrete);
            
            // 3. Wrap in Arc<Arc<Trait>> -> Arc<dyn Any>
            Arc::new(trait_obj) as Arc<dyn Any + Send + Sync>
        });
        
        self.casters.insert(trait_id, caster);
        self
    }

    /// Resolve a service from the container (for concrete types)
    pub fn resolve<T: 'static + Send + Sync>(&self) -> Result<Arc<T>> {
        let requested_type_id = TypeId::of::<T>();

        // Standard resolution
        let entry = self.services.get(&requested_type_id).ok_or_else(|| {
            MeshestraError::DependencyNotFound {
                type_name: std::any::type_name::<T>().to_string(),
            }
        })?;

        // Downcast
        entry
            .instance
            .clone()
            .downcast::<T>()
            .map_err(|_| MeshestraError::DowncastFailed {
                type_name: std::any::type_name::<T>().to_string(),
            })
    }
    
    /// Resolve a trait object from the container
    pub fn resolve_trait<T: ?Sized + 'static + Send + Sync>(&self) -> Result<Arc<T>> {
        let requested_type_id = TypeId::of::<T>();
        
        // Must have a caster
        if let Some(caster) = self.casters.get(&requested_type_id) {
             // Find the implementation type
            let impl_type_id = self.trait_mappings
                .get(&requested_type_id)
                .map(|v| *v)
                .ok_or_else(|| MeshestraError::DependencyNotFound {
                    type_name: std::any::type_name::<T>().to_string(),
                })?;

            // Get concrete instance
            let entry = self.services.get(&impl_type_id).ok_or_else(|| {
                MeshestraError::DependencyNotFound {
                    type_name: std::any::type_name::<T>().to_string(),
                }
            })?;

            // Run caster: returns Arc<Arc<T>> (as Any)
            let cast_result = caster(entry.instance.clone());
            
            // Downcast wrapper to Arc<Arc<T>>
            // T is ?Sized, but Arc<T> is Sized.
            let wrapper = cast_result.downcast::<Arc<T>>().map_err(|_| {
                MeshestraError::DowncastFailed {
                    type_name: "Trait Wrapper Downcast Failed".to_string(),
                }
            })?;
            
            return Ok(wrapper.as_ref().clone());
        }
        
        Err(MeshestraError::DependencyNotFound {
            type_name: "Unknown Trait".to_string(),
        })
    }

    /// Check if a type is registered in the container
    pub fn contains<T: 'static>(&self) -> bool {
        let type_id = TypeId::of::<T>();
        self.services.contains_key(&type_id) || self.trait_mappings.contains_key(&type_id)
    }

    /// Get the number of registered services
    pub fn len(&self) -> usize {
        self.services.len()
    }

    /// Check if the container is empty
    pub fn is_empty(&self) -> bool {
        self.services.is_empty()
    }

    /// Debug: Print all registered services
    #[cfg(debug_assertions)]
    pub fn dump_services(&self) {
        println!("Registered services ({}):", self.services.len());
        for entry in self.services.iter() {
            println!("  - {}", entry.value().type_name);
        }

        println!("\nTrait mappings ({}):", self.trait_mappings.len());
        for mapping in self.trait_mappings.iter() {
            println!("  - {:?} -> {:?}", mapping.key(), mapping.value());
        }
    }
}

impl Default for Container {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestService {
        value: i32,
    }

    #[test]
    fn test_register_and_resolve() {
        let mut container = Container::new();
        container.register(TestService { value: 42 });

        let service = container.resolve::<TestService>().unwrap();
        assert_eq!(service.value, 42);
    }

    #[test]
    fn test_resolve_missing_service() {
        let container = Container::new();
        let result = container.resolve::<TestService>();
        assert!(result.is_err());
    }

    #[test]
    fn test_contains() {
        let mut container = Container::new();
        assert!(!container.contains::<TestService>());

        container.register(TestService { value: 42 });
        assert!(container.contains::<TestService>());
    }
}
