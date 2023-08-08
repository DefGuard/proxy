import './style.scss';

import { useEffect } from 'react';
import { shallow } from 'zustand/shallow';

import { useEnrollmentStore } from '../../hooks/store/useEnrollmentStore';
import { ConfigureDeviceCard } from './components/ConfigureDeviceCard/ConfigureDeviceCard';
import { QuickGuideCard } from './components/QuickGuideCard/QuickGuideCard';

export const DeviceStep = () => {
  const deviceState = useEnrollmentStore((state) => state.deviceState);
  const vpnOptional = useEnrollmentStore((state) => state.vpnOptional);
  const [nextSubject, next] = useEnrollmentStore(
    (state) => [state.nextSubject, state.nextStep],
    shallow,
  );

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
    <div id="enrollment-device-step">
      <ConfigureDeviceCard />
      <QuickGuideCard />
    </div>
  );
};
