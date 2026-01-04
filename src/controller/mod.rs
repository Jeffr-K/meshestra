// Controller functionality is primarily provided through macros:
// - #[controller(path = "...")] for defining controllers
// - #[get], #[post], #[put], #[delete], #[patch] for defining routes
//
// The macros generate:
// 1. Injectable trait implementation for DI
// 2. router() method for Axum integration
