import { createFileRoute, redirect } from '@tanstack/react-router';
import z from 'zod';
import { OpenDesktopPage } from '../pages/OpenDesktop/OpenDesktopPage';

const searchSchema = z.object({
  token: z.string(),
});

export const Route = createFileRoute('/open-desktop')({
  validateSearch: (search) => {
    const parsed = searchSchema.safeParse(search);
    if (!parsed.success) {
      throw redirect({
        to: '/404' as never,
        replace: true,
      });
    }
    return parsed.data;
  },
  component: OpenDesktopPage,
});
