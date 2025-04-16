# Frontend

A Next.js frontend application built with the T3 Stack that provides the user interface for myrtle.moe.

## Table of Contents
- [Features](#features)
- [Prerequisites](#prerequisites)
- [Installation](#installation)
- [Development](#development)
  - [Available Scripts](#available-scripts)
  - [Starting the Development Server](#starting-the-development-server)
- [Project Structure](#project-structure)
- [Configuration](#configuration)
- [Backend Integration](#backend-integration)
- [Deployment](#deployment)
- [Learn More](#learn-more)

## Features

- **[Next.js Framework](https://nextjs.org)**: React framework for building the UI
- **[Tailwind CSS](https://tailwindcss.com)**: Utility-first CSS framework
- **[shadcn/ui](https://ui.shadcn.com/)**: UI component library built with Radix UI and Tailwind
- **[Zustand](https://zustand-demo.pmnd.rs/)**: State management
- **[Framer Motion](https://www.framer.com/motion/)**: Animation library
- **[PixiJS](https://pixijs.com/)**: WebGL rendering for animations

## Prerequisites

- Node.js 18+ or Bun
- Backend service running (see backend README)

## Installation

1. Clone the repository:
```bash
git clone [repository-url]
cd frontend
```

2. Install dependencies:
```bash
npm install
# or
bun install
```

3. Set up environment configuration:
```bash
cp .env.example .env
```

4. Configure the following environment variables in `.env`:
- `BACKEND_URL`: The URL of the backend API (default: "http://localhost:3060")
- `REVALIDATION_SECRET`: Secret key for revalidating static pages

## Development

### Available Scripts

- `npm run dev` - Start the development server
- `npm run build` - Build the production application
- `npm run start` - Start the production server
- `npm run lint` - Run ESLint
- `npm run format:write` - Format code with Prettier

### Starting the Development Server

```bash
npm run dev
# or
bun dev
```

Open [http://localhost:3000](http://localhost:3000) in your browser to view the application.

## Project Structure

```
frontend/
├── src/
│   ├── components/  # Reusable UI components
│   ├── pages/       # Next.js pages and API routes
│   ├── lib/         # Utility functions and shared code
│   ├── styles/      # Global styles and Tailwind configuration
│   ├── hooks/       # Custom React hooks
│   ├── store/       # Zustand stores for state management
│   ├── types/       # TypeScript type definitions
│   └── helper/      # Helper functions
```

## Configuration

The project uses several configuration files:

- `tsconfig.json` - TypeScript configuration
- `.prettierrc` - Code formatting rules
- `.eslintrc.json` - Linting rules
- `.env` - Environment variables

## Backend Integration

The frontend connects to the backend API through the `BACKEND_URL` environment variable. 
By default, this points to `http://localhost:3060` in development.

API requests are handled through custom hooks and service functions that communicate 
with the backend endpoints.

## Deployment

The application can be deployed on platforms like Vercel, Netlify, or using Docker. 
Make sure to set the appropriate environment variables according to your deployment environment.

## Learn More

To learn more about the technologies used in this project:

- [Next.js Documentation](https://nextjs.org/docs)
- [Tailwind CSS Documentation](https://tailwindcss.com/docs)
- [T3 Stack Documentation](https://create.t3.gg/)
