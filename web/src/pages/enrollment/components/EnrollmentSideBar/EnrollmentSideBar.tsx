import './style.scss';

import classNames from 'classnames';
import { useEffect, useMemo, useState } from 'react';
import { LocalizedString } from 'typesafe-i18n';

import { useI18nContext } from '../../../../i18n/i18n-react';
import { Divider } from '../../../../shared/components/layout/Divider/Divider';
import { useApi } from '../../../../shared/hooks/api/useApi.tsx';
import { useEnrollmentStore } from '../../hooks/store/useEnrollmentStore';
import { AdminInfo } from '../AdminInfo/AdminInfo';
import { TimeLeft } from '../TimeLeft/TimeLeft';

export const EnrollmentSideBar = () => {
  const { LL } = useI18nContext();

  const vpnOptional = useEnrollmentStore(
    (state) => state.enrollmentSettings?.vpn_setup_optional,
  );
  const deviceManagementDisabled = useEnrollmentStore((state) =>
    Boolean(
      state.enrollmentSettings?.admin_device_management && !state.userInfo?.is_admin,
    ),
  );
  const [currentStep, stepsMax] = useEnrollmentStore((state) => [
    state.step,
    state.stepsMax,
  ]);
  const enrollmentSettings = useEnrollmentStore((state) => state.enrollmentSettings);

  // fetch app version
  const { getAppInfo } = useApi();
  const [appVersion, setAppVersion] = useState<string | undefined>(undefined);
  useEffect(() => {
    if (!appVersion) {
      getAppInfo()
        .then((res) => {
          setAppVersion(res.version);
        })
        .catch((err) => {
          console.error('Failed to fetch app info: ', err);
        });
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const steps = useMemo((): LocalizedString[] => {
    const stepsLL = LL.pages.enrollment.sideBar.steps;
    const vpnStep = (
      vpnOptional ? `${stepsLL.vpn()}*` : stepsLL.vpn()
    ) as LocalizedString;
    const steps = [
      stepsLL.welcome(),
      stepsLL.verification(),
      stepsLL.password(),
      ...(enrollmentSettings?.only_client_activation || deviceManagementDisabled
        ? [stepsLL.finish()]
        : [vpnStep, stepsLL.finish()]),
    ];
    return steps;
  }, [
    LL.pages.enrollment.sideBar.steps,
    vpnOptional,
    enrollmentSettings,
    deviceManagementDisabled,
  ]);

  return (
    <div id="enrollment-side-bar">
      <div className="title">
        <h2>{LL.pages.enrollment.sideBar.title()}</h2>
      </div>
      <Divider />
      <div className="steps">
        {steps.map((text, index) => (
          <Step text={text} index={index} key={index} />
        ))}
      </div>
      {currentStep !== stepsMax && (
        <>
          <TimeLeft />
          <Divider />
        </>
      )}
      {currentStep === stepsMax && <Divider className="push" />}
      <AdminInfo />
      <Divider />
      <div className="copyright">
        <p>
          Copyright ©{` ${Date.now().getFullYear()} `}
          <a href="https://teonite.com" target="_blank" rel="noopener noreferrer">
            teonite
          </a>
        </p>
        <p>
          {LL.pages.enrollment.sideBar.appVersion()}: {appVersion}
        </p>
      </div>
    </div>
  );
};

type StepProps = {
  text: LocalizedString;
  index: number;
};

const Step = ({ index, text }: StepProps) => {
  const currentStep = useEnrollmentStore((state) => state.step);

  const active = currentStep === index;

  const cn = classNames('step', {
    active,
  });

  return (
    <div className={cn}>
      <p>
        {index + 1}.{'  '}
        {text}
      </p>
      {active && <div className="active-step-line"></div>}
    </div>
  );
};
