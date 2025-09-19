import { createFileRoute } from '@tanstack/react-router';
import { TestPage } from '../test_components/page/TestPage';

export const Route = createFileRoute('/test')({
  component: TestPage,
});
