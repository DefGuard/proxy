import { createFileRoute } from '@tanstack/react-router';
import { HomePage } from '../pages/Home/HomePage';

export const Route = createFileRoute('/')({
  component: HomePage,
});
