# Implementation Plan: F02. Signal-Based Auth State & Route Guard

This plan outlines the steps for creating the Angular Signal-based authentication state provider and routing protection.

## Prerequisites
*   `F01` Landing Page and Clerk SDK wrapper must be fully implemented and verified.

## Phase 1: Authentication State Service
*   **Step 1: Create AuthService** - Implement `AuthService` inside `frontend/src/app/services/auth.service.ts`. The service should inject `ClerkService`, listen to Clerk SDK state changes, and expose reactive Signals for `isAuthenticated`, `user`, and `isLoaded`.
*   **Step 2: Write AuthService Tests** - Add unit tests inside `frontend/src/app/services/auth.service.spec.ts` mocking the Clerk state and validating the output of the Signals.

## Phase 2: Route Guard Protection
*   **Step 3: Create authGuard** - Implement functional `authGuard` inside `frontend/src/app/guards/auth.guard.ts`. The guard will query `AuthService.isAuthenticated()` and redirect unauthorized users to `/login`.
*   **Step 4: Write authGuard Tests** - Implement tests in `frontend/src/app/guards/auth.guard.spec.ts` to mock `AuthService` state and assert correct routing intercepts.
*   **Step 5: Apply Guard to Router Config** - Configure `/dashboard` route inside `frontend/src/app/app.routes.ts` protected by `authGuard` (mock path for dashboard component).
