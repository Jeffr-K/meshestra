use crate::error::{MeshestraError, Result};
use dashmap::DashMap;
use std::any::{Any, TypeId};
use std::sync::Arc;

/// Type alias for a function that can cast an `Arc<dyn Any>` to another `Arc<dyn Any>`.
/// The inner value is usually an `Arc<dyn Trait>`.
type CasterFn = Arc<dyn Fn(Arc<dyn Any + Send + Sync>) -> Arc<dyn Any + Send + Sync> + Send + Sync>;

/// Thread-safe dependency injection container.
pub struct Container {
    services: DashMap<TypeId, ServiceEntry>,
    trait_mappings: DashMap<TypeId, TypeId>,
    casters: DashMap<TypeId, CasterFn>,
}

impl Clone for Container {
    fn clone(&self) -> Self {
        Self {
            services: self.services.clone(),
            trait_mappings: self.trait_mappings.clone(),
            casters: self.casters.clone(),
        }
    }
}

#[derive(Clone)]
struct ServiceEntry {
    instance: Arc<dyn Any + Send + Sync>,
}

impl Container {
    pub fn new() -> Self {
        Self {
            services: DashMap::new(),
            trait_mappings: DashMap::new(),
            casters: DashMap::new(),
        }
    }

    pub fn register<T: 'static + Send + Sync>(&mut self, instance: T) -> &mut Self {
        let type_id = TypeId::of::<T>();
        let entry = ServiceEntry {
            instance: Arc::new(instance),
        };
        self.services.insert(type_id, entry);
        self
    }

    pub fn register_trait<Trait, Impl, F>(&mut self, caster_fn: F) -> &mut Self
    where
        Trait: ?Sized + 'static + Send + Sync,
        Impl: 'static + Send + Sync,
        F: Fn(Arc<Impl>) -> Arc<Trait> + 'static + Send + Sync,
    {
        let trait_id = TypeId::of::<Trait>();
        let impl_id = TypeId::of::<Impl>();

        self.trait_mappings.insert(trait_id, impl_id);

        let caster: CasterFn = Arc::new(move |instance: Arc<dyn Any + Send + Sync>| {
            let concrete = instance
                .downcast::<Impl>()
                .expect("Failed to downcast to implementation type. This is a bug in Meshestra.");
            let trait_obj: Arc<Trait> = caster_fn(concrete);
            Arc::new(trait_obj) // Wrap the Arc<dyn Trait> in an Arc<dyn Any>
        });

        self.casters.insert(trait_id, caster);
        self
    }

    pub fn resolve<T: 'static + Send + Sync>(&self) -> Result<Arc<T>> {
        let requested_type_id = TypeId::of::<T>();
        let entry = self.services.get(&requested_type_id).ok_or_else(|| {
            MeshestraError::DependencyNotFound {
                type_name: std::any::type_name::<T>().to_string(),
            }
        })?;
        entry
            .instance
            .clone()
            .downcast::<T>()
            .map_err(|_| MeshestraError::DowncastFailed {
                type_name: std::any::type_name::<T>().to_string(),
            })
    }

    pub fn resolve_trait<T: ?Sized + 'static + Send + Sync>(&self) -> Result<Arc<T>> {
        let requested_type_id = TypeId::of::<T>();

        let caster = self.casters.get(&requested_type_id).ok_or_else(|| {
            MeshestraError::DependencyNotFound {
                type_name: std::any::type_name::<T>().to_string(),
            }
        })?;

        let impl_type_id = self.trait_mappings.get(&requested_type_id).ok_or_else(|| {
            MeshestraError::DependencyNotFound {
                type_name: format!(
                    "No implementation mapping found for trait '{}'",
                    std::any::type_name::<T>()
                ),
            }
        })?;

        let entry =
            self.services
                .get(&impl_type_id)
                .ok_or_else(|| MeshestraError::DependencyNotFound {
                    type_name: format!(
                        "Implementation for trait '{}' not registered",
                        std::any::type_name::<T>()
                    ),
                })?;

        let cast_result = (caster.value())(entry.instance.clone());

        // The caster returns an Arc<dyn Any> which holds an Arc<T>.
        // We need to downcast to Arc<T>, which is Sized.
        let wrapper =
            cast_result
                .downcast::<Arc<T>>()
                .map_err(|_| MeshestraError::DowncastFailed {
                    type_name: format!(
                        "Failed to downcast to Arc<Arc<{}>>. This is an internal Meshestra bug.",
                        std::any::type_name::<T>()
                    ),
                })?;
        // The result of downcast is Arc<Arc<T>>, so we clone the inner Arc.
        Ok(wrapper.as_ref().clone())
    }

    pub fn contains<T: 'static>(&self) -> bool {
        let type_id = TypeId::of::<T>();
        self.services.contains_key(&type_id) || self.trait_mappings.contains_key(&type_id)
    }

    pub fn len(&self) -> usize {
        self.services.len()
    }

    pub fn is_empty(&self) -> bool {
        self.services.is_empty()
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

    trait MyTrait: Send + Sync {
        fn get_value(&self) -> i32;
    }

    struct MyTraitImpl {
        value: i32,
    }

    impl MyTrait for MyTraitImpl {
        fn get_value(&self) -> i32 {
            self.value
        }
    }

    #[test]
    fn test_register_and_resolve() {
        let mut container = Container::new();
        container.register(TestService { value: 42 });
        let service = container.resolve::<TestService>().unwrap();
        assert_eq!(service.value, 42);
    }

    #[test]
    fn test_register_and_resolve_trait() {
        let mut container = Container::new();
        container.register(MyTraitImpl { value: 99 });
        container.register_trait::<dyn MyTrait, MyTraitImpl, _>(|i| i as Arc<dyn MyTrait>);
        let trait_instance = container.resolve_trait::<dyn MyTrait>().unwrap();
        assert_eq!(trait_instance.get_value(), 99);
    }
}
