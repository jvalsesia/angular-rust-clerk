ARG SERVICE_PATH=.

# Stage 1: Build the Angular application
FROM node:20-alpine AS build
ARG SERVICE_PATH
WORKDIR /app
COPY ${SERVICE_PATH}/package*.json ./
RUN npm ci
COPY ${SERVICE_PATH}/. .
RUN npm run build

# Stage 2: Serve the application with Nginx
FROM nginx:alpine
ARG SERVICE_PATH
COPY --from=build /app/dist/frontend/browser /usr/share/nginx/html
COPY ${SERVICE_PATH}/nginx.conf /etc/nginx/conf.d/default.conf
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
