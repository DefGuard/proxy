import { createFileRoute } from '@tanstack/react-router';
import { PasswordStartPage } from '../../pages/PasswordStart/PasswordStartPage';

export const Route = createFileRoute('/password/')({
  component: PasswordStartPage,
});
