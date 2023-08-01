import './style.scss';

import { useI18nContext } from '../../../../../../i18n/i18n-react';
import { Card } from '../../../../../../shared/components/layout/Card/Card';
import { EnrollmentStepIndicator } from '../../../../components/EnrollmentStepIndicator/EnrollmentStepIndicator';
import { CreateDevice } from './components/CreateDevice';

export const ConfigureDeviceCard = () => {
  const { LL } = useI18nContext();

  const cardLL = LL.pages.enrollment.steps.deviceSetup.cards.device;

  return (
    <Card id="configure-device-card">
      <EnrollmentStepIndicator />
      <h3>{cardLL.title()}</h3>
      <CreateDevice />
    </Card>
  );
};
