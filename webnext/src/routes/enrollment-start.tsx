import { createFileRoute } from '@tanstack/react-router';
import { EnrollmentStartPage } from '../pages/enrollment/EnrollmentStart/EnrollmentStartPage';

export const Route = createFileRoute('/enrollment-start')({
  component: EnrollmentStartPage,
});
