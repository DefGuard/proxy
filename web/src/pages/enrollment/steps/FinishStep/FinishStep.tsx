import './style.scss';

import { useI18nContext } from '../../../../i18n/i18n-react';
import { Card } from '../../../../shared/components/layout/Card/Card';
import { EnrollmentStepIndicator } from '../../components/EnrollmentStepIndicator/EnrollmentStepIndicator';

export const FinishStep = () => {
  const { LL } = useI18nContext();

  return (
    <Card id="enrollment-finish-card">
      <EnrollmentStepIndicator />
      <h3>{LL.pages.enrollment.steps.finish.title()}</h3>
    </Card>
  );
};
