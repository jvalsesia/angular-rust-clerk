# Implementation Plan: F01. Landing Page & Clerk Auth UI

**Prerequisites:**
- Node.js version 22+
- Angular CLI version 19+ (installed globally or run via npx)
- Clerk Publishable Key (obtained from the Clerk dashboard)

### Stage 1: Workspace & Workspace Config

**1. Scaffolding Angular Frontend** - Create the `frontend/` directory structure using the Angular CLI. Clean up template files and configure compiler options to target Vite and standard SPA routing.

**2. Zoneless Configuration** - Update the core application configuration to replace Zone.js change detection with experimental zoneless change detection. Disable zone.js imports in configuration files.

**3. Angular Material Integration** - Add Angular Material dependencies and configure theme styling files, registering default fonts and loading animations.

### Stage 2: Routing & Clerk Integration

**4. Route Layout Scaffolding** - Define root routes linking `/` to the landing component, `/login` to the login component, and `/register` to the registration component, including layout placeholders.

**5. Clerk SDK Integration Service** - Implement a service that dynamically loads Clerk's JavaScript SPA library, initializes it with the Clerk publishable key, and provides methods to mount sign-in/sign-up forms.

**6. Login & Signup Views** - Implement wrapper components that leverage the Clerk service to render authentication widgets in the browser, showing loading screens until the Clerk scripts are ready.
