import { createFileRoute } from '@tanstack/react-router';
import z from 'zod';
import { OpenDesktopPage } from '../pages/OpenDesktop/OpenDesktopPage';

const searchSchema = z.object({
  token: z.string().optional(),
});

export const Route = createFileRoute('/open-desktop')({
  validateSearch: searchSchema,
  component: OpenDesktopPage,
});
