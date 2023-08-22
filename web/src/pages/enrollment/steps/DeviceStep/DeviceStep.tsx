import './style.scss';

import classNames from 'classnames';
import { useEffect } from 'react';
import { shallow } from 'zustand/shallow';

import { useI18nContext } from '../../../../i18n/i18n-react';
import { MessageBox } from '../../../../shared/components/layout/MessageBox/MessageBox';
import { MessageBoxType } from '../../../../shared/components/layout/MessageBox/types';
import { useEnrollmentStore } from '../../hooks/store/useEnrollmentStore';
import { ConfigureDeviceCard } from './components/ConfigureDeviceCard/ConfigureDeviceCard';
import { QuickGuideCard } from './components/QuickGuideCard/QuickGuideCard';

export const DeviceStep = () => {
  const { LL } = useI18nContext();
  const deviceState = useEnrollmentStore((state) => state.deviceState);
  const vpnOptional = useEnrollmentStore((state) => state.vpnOptional);
  const [nextSubject, next] = useEnrollmentStore(
    (state) => [state.nextSubject, state.nextStep],
    shallow,
  );

  const cn = classNames({
    required: !vpnOptional,
    optional: vpnOptional,
  });

  useEffect(() => {
    const sub = nextSubject.subscribe(() => {
      if ((deviceState && deviceState.device && deviceState.configs) || vpnOptional) {
        next();
      }
    });

    return () => {
      sub.unsubscribe();
    };
  }, [deviceState, next, nextSubject, vpnOptional]);

  return (
    <div id="enrollment-device-step" className={cn}>
      {vpnOptional && (
        <MessageBox
          type={MessageBoxType.WARNING}
          message={LL.pages.enrollment.steps.deviceSetup.optionalMessage()}
        />
      )}
      <div className="cards">
        <ConfigureDeviceCard />
        <QuickGuideCard />
      </div>
    </div>
  );
};
