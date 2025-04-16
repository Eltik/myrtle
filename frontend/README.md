# Frontend

This is a Next.js frontend application built with the T3 Stack that provides the user interface for myrtle.moe.

## Technology Stack

- [Next.js](https://nextjs.org) - React framework for building the UI
- [Tailwind CSS](https://tailwindcss.com) - Utility-first CSS framework
- [shadcn/ui](https://ui.shadcn.com/) - UI component library built with Radix UI and Tailwind
- [Zustand](https://zustand-demo.pmnd.rs/) - State management
- [Framer Motion](https://www.framer.com/motion/) - Animation library
- [PixiJS](https://pixijs.com/) - WebGL rendering for animations

## Getting Started

### Prerequisites

- Node.js 18+ or Bun
- Backend service running (see backend README)

### Setup Instructions

1. Clone the repository
2. Navigate to the frontend directory
   ```bash
   cd frontend
   ```
3. Install dependencies
   ```bash
   npm install
   # or
   bun install
   ```
4. Copy the environment variables
   ```bash
   cp .env.example .env
   ```
5. Update the `.env` file with:
   ```
   BACKEND_URL="http://localhost:3060"
   REVALIDATION_SECRET="your-secret-here"
   ```
6. Start the development server
   ```bash
   npm run dev
   # or
   bun dev
   ```
7. Open [http://localhost:3000](http://localhost:3000) in your browser

## Project Structure

- `src/components/` - Reusable UI components
- `src/pages/` - Next.js pages and API routes
- `src/lib/` - Utility functions and shared code
- `src/styles/` - Global styles and Tailwind configuration
- `src/hooks/` - Custom React hooks
- `src/store/` - Zustand stores for state management
- `src/types/` - TypeScript type definitions
- `src/helper/` - Helper functions

## Connecting to the Backend

The frontend connects to the backend API through the `BACKEND_URL` environment variable. By default, this points to `http://localhost:3060` in development.

API requests are handled through custom hooks and service functions that communicate with the backend endpoints.

### Environment Variables

- `BACKEND_URL` - The URL of the backend API
- `REVALIDATION_SECRET` - Secret key for revalidating static pages

## Scripts

- `npm run dev` - Start the development server
- `npm run build` - Build the production application
- `npm run start` - Start the production server
- `npm run lint` - Run ESLint
- `npm run format:write` - Format code with Prettier

## Deployment

The application can be deployed on platforms like Vercel, Netlify, or using Docker. Make sure to set the appropriate environment variables according to your deployment environment.

## Learn More

To learn more about the technologies used in this project:

- [Next.js Documentation](https://nextjs.org/docs)
- [Tailwind CSS Documentation](https://tailwindcss.com/docs)
- [T3 Stack Documentation](https://create.t3.gg/)
