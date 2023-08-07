import './style.scss';

import classNames from 'classnames';
import { useMemo } from 'react';
import { LocalizedString } from 'typesafe-i18n';

import { useI18nContext } from '../../../../i18n/i18n-react';
import { Divider } from '../../../../shared/components/layout/Divider/Divider';
import { useEnrollmentStore } from '../../hooks/store/useEnrollmentStore';
import { AdminInfo } from '../AdminInfo/AdminInfo';
import { TimeLeft } from '../TimeLeft/TimeLeft';

export const EnrollmentSideBar = () => {
  const { LL } = useI18nContext();

  const steps = useMemo((): LocalizedString[] => {
    const steps = LL.pages.enrollment.sideBar.steps;
    return [
      steps.welcome(),
      steps.verification(),
      steps.password(),
      steps.vpn(),
      steps.finish(),
    ];
  }, [LL.pages.enrollment.sideBar.steps]);

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
      <TimeLeft />
      <Divider />
      <AdminInfo />
      <Divider />
      <div className="copyright">
        <p>
          Copyright © 2023{' '}
          <a href="https://teonite.com" target="_blank" rel="noopener noreferrer">
            teonite
          </a>
        </p>
        <p>{LL.pages.enrollment.sideBar.appVersion()}: 0.4.1</p>
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
