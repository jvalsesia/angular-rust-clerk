# Implementation Plan: F04. Authenticated HTTP Client Interceptor

This plan covers building the Angular HTTP Interceptor, writing tests, and registering it in the main application config.

## Prerequisites
- `F02` Signal-Based Auth State & Route Guard must be complete.

## Phase 1: HTTP Interceptor
- **Step 1: Create Interceptor** - Implement `authInterceptor` in `frontend/src/app/interceptors/auth.interceptor.ts`. The interceptor will filter URLs containing `/api/`, call `AuthService.getToken()`, clone the request, and attach the Bearer header.
- **Step 2: Write Interceptor Tests** - Add unit tests inside `frontend/src/app/interceptors/auth.interceptor.spec.ts` using `HttpTestingController` to verify request modification.

## Phase 2: App Configuration Integration
- **Step 3: Register Interceptor** - Update `frontend/src/app/app.config.ts` to include `provideHttpClient(withInterceptors([authInterceptor]))` in the providers array.
