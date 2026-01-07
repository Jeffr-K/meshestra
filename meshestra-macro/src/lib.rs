use proc_macro::TokenStream;

mod aspect;
mod controller;
mod exception;
mod http_methods;
mod injectable;
mod interceptor;
mod module;
mod transactional;

/// Derive macro for making a struct injectable into the DI container
///
/// # Example
/// ```
/// use meshestra::Injectable;
///
/// #[derive(Injectable)]
/// pub struct UserService {
///     repository: Arc<dyn UserRepository>,
/// }
/// ```
#[proc_macro_derive(Injectable)]
pub fn derive_injectable(input: TokenStream) -> TokenStream {
    injectable::derive_injectable(input)
}

/// Attribute macro for defining a controller with automatic DI registration
///
/// # Example
/// ```
/// use meshestra::controller;
///
/// #[controller(path = "/users")]
/// pub struct UserController {
///     user_service: Arc<UserService>,
/// }
///
/// impl UserController {
///     #[get("/{id}")]
///     async fn get_user(&self, Path(id): Path<String>) -> Response {
///         // ...
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn controller(attr: TokenStream, item: TokenStream) -> TokenStream {
    controller::controller_attribute(attr, item)
}

/// Attribute macro for defining routes in an impl block
///
/// # Example
/// ```
/// #[routes(UserController)]
/// impl UserController {
///     #[get("/:id")]
///     async fn get_user(&self, Path(id): Path<String>) -> Json<User> {
///         // ...
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn routes(attr: TokenStream, item: TokenStream) -> TokenStream {
    controller::routes_attribute(attr, item)
}

/// Attribute macro for defining a module with providers and controllers
///
/// # Example
/// ```
/// use meshestra::module;
///
/// #[module(
///     controllers = [UserController],
///     providers = [UserService, UserRepositoryImpl],
/// )]
/// pub struct AppModule;
/// ```
#[proc_macro_attribute]
pub fn module(attr: TokenStream, item: TokenStream) -> TokenStream {
    module::module_attribute(attr, item)
}

/// Attribute macro for defining interceptors on a controller
///
/// # Example
/// ```
/// #[interceptor(LoggingInterceptor)]
/// pub struct UserController { ... }
/// ```
#[proc_macro_attribute]
pub fn interceptor(attr: TokenStream, item: TokenStream) -> TokenStream {
    interceptor::interceptor_attribute(attr, item)
}

/// Wrapped an async function to execute within a transaction
///
/// # Example
/// ```
/// #[transactional]
/// async fn create_user(&self, user: User) -> Result<User> { ... }
/// ```
#[proc_macro_attribute]
pub fn transactional(attr: TokenStream, item: TokenStream) -> TokenStream {
    transactional::transactional_attribute(attr, item)
}

/// Attribute macro for defining an exception filter
///
/// # Example
/// ```
/// #[exception_filter]
/// pub struct GlobalExceptionFilter;
/// ```
#[proc_macro_attribute]
pub fn exception_filter(attr: TokenStream, item: TokenStream) -> TokenStream {
    exception::exception_filter_attribute(attr, item)
}

/// Attribute macro for defining an exception handler method
///
/// # Example
/// ```
/// #[handle(UserError)]
/// fn handle_user(&self, err: UserError) -> Response { ... }
/// ```
#[proc_macro_attribute]
pub fn handle(attr: TokenStream, item: TokenStream) -> TokenStream {
    exception::handle_attribute(attr, item)
}

/// HTTP GET method attribute for controller methods
#[proc_macro_attribute]
pub fn get(attr: TokenStream, item: TokenStream) -> TokenStream {
    http_methods::http_method_attribute("GET", attr, item)
}

/// HTTP POST method attribute for controller methods
#[proc_macro_attribute]
pub fn post(attr: TokenStream, item: TokenStream) -> TokenStream {
    http_methods::http_method_attribute("POST", attr, item)
}

/// HTTP PUT method attribute for controller methods
#[proc_macro_attribute]
pub fn put(attr: TokenStream, item: TokenStream) -> TokenStream {
    http_methods::http_method_attribute("PUT", attr, item)
}

/// HTTP DELETE method attribute for controller methods
#[proc_macro_attribute]
pub fn delete(attr: TokenStream, item: TokenStream) -> TokenStream {
    http_methods::http_method_attribute("DELETE", attr, item)
}

/// HTTP PATCH method attribute for controller methods
#[proc_macro_attribute]
pub fn patch(attr: TokenStream, item: TokenStream) -> TokenStream {
    http_methods::http_method_attribute("PATCH", attr, item)
}

/// Parameter attribute for request body (JSON)
/// Wraps the parameter with axum::Json extractor
#[proc_macro_attribute]
pub fn body(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Pass-through, actual handling is done by #[routes] macro
    item
}

/// Parameter attribute for path parameters
/// Wraps the parameter with axum::extract::Path extractor
#[proc_macro_attribute]
pub fn param(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Pass-through, actual handling is done by #[routes] macro
    item
}

/// Parameter attribute for query string parameters
/// Wraps the parameter with axum::extract::Query extractor
#[proc_macro_attribute]
pub fn query(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Pass-through, actual handling is done by #[routes] macro
    item
}

/// Parameter attribute for request headers
/// Wraps the parameter with axum::extract::Header extractor
///
/// # Example
/// ```
/// #[headers(param?: string)]
/// impl UserController {
///     #[get("/")]
///     async fn get_user(&self, #[header] h: Header<String>) -> Response {
///         // ...
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn headers(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Pass-through, actual handling is done by #[routes] macro
    item
}

/// Parameter attribute for request IP address
/// Wraps the parameter with axum::extract::ConnectInfo extractor
///
/// # Example
/// ```
/// impl UserController {
///     #[get("/")]
///     async fn get_user(&self, #[ip] ip: String) -> Response {
///         // ...
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn ip(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Pass-through, actual handling is done by #[routes] macro
    item
}

/// Parameter attribute for request host
/// Wraps the parameter with axum::extract::ConnectInfo extractor
///
/// # Example
/// ```
/// impl UserController {
///     #[get("/")]
///     async fn get_user(&self, #[host_param] host: String) -> Response {
///         // ...
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn host_param(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Pass-through, actual handling is done by #[routes] macro
    item
}

/// Attribute macro for applying Cross-Cutting Concerns (Aspects) to a controller or specific methods.
///
/// # Description
/// Aspects allow you to inject logic (Before/After/On Error) around your handler execution.
/// You can apply this to an entire `struct` (controller) to affect all its routes,
/// or to individual `async fn` methods for fine-grained control.
///
/// # Requirements
/// The type passed to the attribute (e.g., `LoggingAspect`) must:
/// 1. Implement the `Aspect` trait.
/// 2. Be registered as a `provider` in a `Module`.
/// 3. Be `Injectable` or available in the DI container.
///
/// # Execution Order
/// 1. `Aspect::before`: Runs before the handler. Can abort execution by returning `Err`.
/// 2. `Handler`: Your actual controller method.
/// 3. `Aspect::after`: Runs after successful handler execution.
/// 4. `Aspect::on_error`: Runs if the handler or interceptors fail.
///
/// # Example
/// ```rust
/// // Apply to all methods in the controller
/// #[controller(path = "/users")]
/// #[aspect(AuthAspect)]
/// pub struct UserController { ... }
///
/// impl UserController {
///     // Apply to a specific method (runs after controller-level aspects)
///     #[get("/:id")]
///     #[aspect(LoggingAspect)]
///     async fn get_user(&self, #[param] id: String) -> ApiResponse<User> {
///         // ...
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn aspect(attr: TokenStream, item: TokenStream) -> TokenStream {
    aspect::aspect_attribute(attr, item)
}
