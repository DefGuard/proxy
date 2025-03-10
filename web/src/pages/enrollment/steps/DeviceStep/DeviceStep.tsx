import './style.scss';

import { useMutation } from '@tanstack/react-query';
import { AxiosError } from 'axios';
import classNames from 'classnames';
import { useEffect } from 'react';
import { shallow } from 'zustand/shallow';

import { useI18nContext } from '../../../../i18n/i18n-react';
import { LoaderSpinner } from '../../../../shared/components/layout/LoaderSpinner/LoaderSpinner';
import { MessageBox } from '../../../../shared/components/layout/MessageBox/MessageBox';
import { MessageBoxType } from '../../../../shared/components/layout/MessageBox/types';
import { useApi } from '../../../../shared/hooks/api/useApi';
import useEffectOnce from '../../../../shared/hooks/api/utils';
import { useEnrollmentStore } from '../../hooks/store/useEnrollmentStore';
import { ConfigureDeviceCard } from './components/ConfigureDeviceCard/ConfigureDeviceCard';
import { QuickGuideCard } from './components/QuickGuideCard/QuickGuideCard';

export const DeviceStep = () => {
  const {
    enrollment: { activateUser },
  } = useApi();
  const { LL } = useI18nContext();
  const setStore = useEnrollmentStore((state) => state.setState);
  const deviceState = useEnrollmentStore((state) => state.deviceState);
  const settings = useEnrollmentStore((state) => state.enrollmentSettings);
  const [userPhone, userPassword] = useEnrollmentStore(
    (state) => [state.userInfo?.phone_number, state.userPassword],
    shallow,
  );
  const [nextSubject, next] = useEnrollmentStore(
    (state) => [state.nextSubject, state.nextStep],
    shallow,
  );

  const cn = classNames({
    required: !settings?.vpn_setup_optional,
    optional: settings?.vpn_setup_optional,
  });

  const { mutate } = useMutation({
    mutationFn: activateUser,
    onSuccess: () => {
      setStore({ loading: false });
      next();
    },
    onError: (err: AxiosError) => {
      setStore({ loading: false });
      console.error(err.message);
    },
  });

  useEffect(() => {
    if (userPassword) {
      const sub = nextSubject.subscribe(() => {
        if (
          (deviceState && deviceState.device && deviceState.configs) ||
          settings?.vpn_setup_optional ||
          settings?.only_client_activation
        ) {
          setStore({
            loading: true,
          });
          mutate({
            password: userPassword,
            phone_number: userPhone,
          });
        }
      });

      return () => {
        sub.unsubscribe();
      };
    }
  }, [
    deviceState,
    nextSubject,
    settings?.vpn_setup_optional,
    setStore,
    userPhone,
    userPassword,
    mutate,
    settings?.only_client_activation,
  ]);

  // If only client activation is enabled, skip manual wireguard setup
  useEffectOnce(() => {
    if (settings?.only_client_activation) {
      nextSubject.next();
    }
  });

  return (
    <div id="enrollment-device-step" className={cn}>
      {!settings?.only_client_activation ? (
        <>
          {settings?.vpn_setup_optional && (
            <MessageBox
              type={MessageBoxType.WARNING}
              message={LL.pages.enrollment.steps.deviceSetup.optionalMessage()}
            />
          )}
          <div className="cards">
            <ConfigureDeviceCard />
            <QuickGuideCard />
          </div>
        </>
      ) : (
        <div id="loader">
          <LoaderSpinner size={80} />
        </div>
      )}
    </div>
  );
};
