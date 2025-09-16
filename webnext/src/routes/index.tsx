import { createFileRoute } from '@tanstack/react-router';
import { IndexPage } from '../pages/IndexPage/IndexPage';

export const Route = createFileRoute('/')({
  component: IndexPage,
});
