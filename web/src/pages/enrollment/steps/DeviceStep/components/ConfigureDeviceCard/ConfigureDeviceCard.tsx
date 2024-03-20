import './style.scss';

import { isUndefined } from 'lodash-es';

import { useI18nContext } from '../../../../../../i18n/i18n-react';
import { Card } from '../../../../../../shared/components/layout/Card/Card';
import { MessageBox } from '../../../../../../shared/components/layout/MessageBox/MessageBox';
import { MessageBoxType } from '../../../../../../shared/components/layout/MessageBox/types';
import { EnrollmentStepIndicator } from '../../../../components/EnrollmentStepIndicator/EnrollmentStepIndicator';
import { useEnrollmentStore } from '../../../../hooks/store/useEnrollmentStore';
import { CreateDevice } from './components/CreateDevice';
import { DeviceConfiguration } from './components/DeviceConfiguration/DeviceConfiguration';

export const ConfigureDeviceCard = () => {
  const { LL } = useI18nContext();

  const deviceState = useEnrollmentStore((state) => state.deviceState);

  // TODO
  const networksAvailable = false;
  const configAvailable =
    deviceState &&
    !isUndefined(deviceState?.device) &&
    !isUndefined(deviceState?.configs);

  const cardLL = LL.pages.enrollment.steps.deviceSetup.cards.device;

  return (
    <Card id="configure-device-card">
      <EnrollmentStepIndicator />
      <h3>{cardLL.title()}</h3>
      {networksAvailable && !configAvailable && <CreateDevice />}
      {networksAvailable && configAvailable && <DeviceConfiguration />}
      {!networksAvailable && (
        <MessageBox message={cardLL.noNetworksMessage()} type={MessageBoxType.WARNING} />
      )}
    </Card>
  );
};
