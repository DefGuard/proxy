import { createFileRoute } from '@tanstack/react-router';
import { PasswordFormPage } from '../../pages/PasswordForm/PasswordFormPage';

export const Route = createFileRoute('/password/form')({
  component: PasswordFormPage,
});
