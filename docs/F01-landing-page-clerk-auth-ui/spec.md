# Technical Specification: F01. Landing Page & Clerk Auth UI

## 1. Technical Overview

This feature establishes the frontend workspace for the boilerplate template by bootstrapping an Angular 21+ application within the monorepo's `frontend/` directory. The application will be configured to use Vite as the modern build pipeline and compilation engine, and Vitest as the unit test runner. To maximize performance and leverage contemporary Angular practices, the application will run in a fully zoneless reactivity mode.

Authentication UI workflows (identity creation, user validation, sign-ins, and sign-ups) are offloaded to Clerk's hosted Identity-as-a-Service (IDaaS) platform. This feature integrates the Clerk JavaScript SPA SDK into the Angular environment, creating dedicated routes and components to host the standard embedded Clerk Login and Registration widgets.

### Scope

**Included:**
*   Angular 21+ project scaffolding in the `frontend/` directory using modern configuration defaults.
*   Setup of `provideExperimentalZonelessChangeDetection()` in the main application bootstrap config to eliminate `zone.js`.
*   Routes mapping `/` to a landing page, `/login` to a login screen, and `/register` to a sign-up screen.
*   Implementation of a dynamic script loader service to fetch and initialize the Clerk JS SDK (`@clerk/clerk-js`).
*   Embedded UI component wrappers for Clerk's `<clerk-sign-in>` and `<clerk-sign-up>` modal forms.
*   Material loading spinners to prevent cumulative layout shifts while Clerk elements load.

**Excluded:**
*   Signal-based auth state synchronization (e.g., exposing reactive `isAuthenticated` signals), which is covered under `F02`.
*   Functional Route Guards to gate access to the `/dashboard` route (covered under `F02`).
*   HTTP request interception to automatically attach JWT authorization headers (covered under `F04`).

## 2. Architecture Impact

### Affected Components

The following components and configuration files will be created in the `frontend/` workspace:
*   `frontend/angular.json`
*   `frontend/package.json`
*   `frontend/src/main.ts`
*   `frontend/src/app/app.config.ts`
*   `frontend/src/app/app.routes.ts`
*   `frontend/src/app/services/clerk.service.ts`
*   `frontend/src/app/components/landing/landing.component.ts`
*   `frontend/src/app/components/login/login.component.ts`
*   `frontend/src/app/components/register/register.component.ts`

### Data Flow Diagram

```mermaid
graph TD
    User -->|Access "/"| Landing["LandingComponent"]
    User -->|Access "/login"| Login["LoginComponent"]
    User -->|Access "/register"| Register["RegisterComponent"]
    
    Login -->|Calls mount| ClerkService["ClerkService"]
    Register -->|Calls mount| ClerkService
    
    ClerkService -->|Loads SDK Script| ClerkCDN["Clerk SDK (CDN)"]
    ClerkCDN -->|Renders Form| ClerkUI["Clerk Embedded UI Container"]
    
    ClerkUI -->|Auth Success| AuthRedirect["Redirect to /dashboard"]
```

## 3. Technical Decisions

| Decision | Chosen Approach | Alternative Considered | Trade-off |
|----------|----------------|----------------------|-----------|
| **Framework Scaffolding** | Angular 21 via standard application builder (Vite-backed) | Legacy Webpack builder configuration | Alignment with the latest Angular standards, reducing bundle sizes and build compilation times at the expense of compatibility with obsolete Angular libraries. |
| **Reactivity Paradigm** | Zoneless Change Detection (`provideExperimentalZonelessChangeDetection()`) | Zone.js-based automatic change detection | Eliminates the standard ~100KB runtime size overhead of `zone.js` and speeds up rendering cycles, but requires explicit template logic handling (Signals or async pipes). |
| **Clerk SDK Wiring** | Custom Angular service (`ClerkService`) importing `@clerk/clerk-js` | Third-party or community Clerk-Angular wrappers | Direct utilization of Clerk's official JavaScript SDK ensures long-term version compatibility and provides control over script load lifecycles, bypassing unmaintained community dependencies. |
| **Widget Mounting** | Dedicated route wrappers with inline targets (`#clerkSignIn`) | Global overlay modals or Clerk Hosted accounts | Keeps the user inside the application domain during authentication. Renders within our viewport rather than redirecting to a Clerk subdomain. |

## 4. Component Overview

| File Path | New/Modified | Purpose | Key Responsibilities |
|-----------|--------------|---------|---------------------|
| `frontend/src/main.ts` | New | Application Entry Point | Bootstraps the root component using the zoneless configuration setup. |
| `frontend/src/app/app.config.ts` | New | Application Configuration | Configures core routing, zoneless change detection, and basic HTTP providers. |
| `frontend/src/app/app.routes.ts` | New | Router Mapping | Defines path maps matching landing, login, signup, and dashboard placeholder routes. |
| `frontend/src/app/services/clerk.service.ts` | New | Clerk SDK Lifecycle Service | Dynamically injects the Clerk script loader, initializes the SDK, and mounts Clerk UI widgets onto target HTML elements. |
| `frontend/src/app/components/landing/landing.component.ts` | New | Landing Page Component | Displays public welcome copy, product value statements, and navigation buttons. |
| `frontend/src/app/components/login/login.component.ts` | New | Login Component | Houses the HTML target node for the Clerk Sign-In widget and handles loading states. |
| `frontend/src/app/components/register/register.component.ts` | New | Registration Component | Houses the HTML target node for the Clerk Sign-Up widget and handles loading states. |

## 5. API Contracts

*This feature is purely frontend scaffolding and contains no direct backend endpoints.*

## 6. Data Model

*This feature does not interact with or define database structures.*

## 7. Testing Strategy

### Test Layout

| Test File | Test Type | Target | Coverage Goal |
|-----------|-----------|--------|---------------|
| `frontend/src/app/components/landing/landing.component.spec.ts` | Unit | LandingComponent | 90% |
| `frontend/src/app/components/login/login.component.spec.ts` | Unit | LoginComponent | 80% |
| `frontend/src/app/services/clerk.service.spec.ts` | Unit | ClerkService | 85% |

### Test Specifications

| Test Function | Description | Assertions |
|---------------|-------------|------------|
| `shouldCreateLanding` | Validates landing component initialization. | Component instance is truthy. |
| `shouldRenderGetStartedButton` | Confirms landing page renders the primary navigation button. | Button element with routing target exists. |
| `shouldLoadClerkScript` | Verifies the Clerk script tag is appended to document head on init. | Script tag with Clerk CDN URL is present in DOM. |
| `shouldMountSignInWidget` | Confirms that the sign-in widget mounting logic matches. | Service passes correct element reference to Clerk mount method. |
| `shouldDisplayLoader` | Verifies that loading spinner displays while Clerk SDK is active. | Material progress spinner is visible when loading signal is true. |
