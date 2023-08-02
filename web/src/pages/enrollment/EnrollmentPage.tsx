import './style.scss';

import dayjs from 'dayjs';
import { ReactNode, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { useBreakpoint } from 'use-breakpoint';
import { shallow } from 'zustand/shallow';

import { LogoContainer } from '../../components/LogoContainer/LogoContainer';
import { useI18nContext } from '../../i18n/i18n-react';
import { ArrowSingle } from '../../shared/components/icons/ArrowSingle/ArrowSingle';
import {
  ArrowSingleDirection,
  ArrowSingleSize,
} from '../../shared/components/icons/ArrowSingle/types';
import { Button } from '../../shared/components/layout/Button/Button';
import {
  ButtonSize,
  ButtonStyleVariant,
} from '../../shared/components/layout/Button/types';
import { PageContainer } from '../../shared/components/layout/PageContainer/PageContainer';
import { deviceBreakpoints } from '../../shared/constants';
import { routes } from '../../shared/routes';
import { EnrollmentSideBar } from './components/EnrollmentSideBar/EnrollmentSideBar';
import { EnrollmentStepControls } from './components/EnrollmentStepControls/EnrollmentStepControls';
import { useEnrollmentStore } from './hooks/store/useEnrollmentStore';
import { DataVerificationStep } from './steps/DataVerificationStep/DataVerificationStep';
import { DeviceStep } from './steps/DeviceStep/DeviceStep';
import { FinishStep } from './steps/FinishStep/FinishStep';
import { PasswordStep } from './steps/PasswordStep/PasswordStep';
import { WelcomeStep } from './steps/WelcomeStep/WelcomeStep';

export const EnrollmentPage = () => {
  const navigate = useNavigate();
  const { LL } = useI18nContext();
  const { breakpoint } = useBreakpoint(deviceBreakpoints);
  const sessionEnd = useEnrollmentStore((state) => state.sessionEnd);
  const currentStep = useEnrollmentStore((state) => state.step);
  const stepsMax = useEnrollmentStore((state) => state.stepsMax);

  const [setEnrollmentState, back, reset, nextSubject] = useEnrollmentStore(
    (state) => [state.setState, state.perviousStep, state.reset, state.nextSubject],
    shallow,
  );

  const controlsSize: ButtonSize =
    breakpoint !== 'desktop' ? ButtonSize.SMALL : ButtonSize.LARGE;

  // ensure number of steps is correct
  useEffect(() => {
    if (stepsMax !== steps.length - 1) {
      setEnrollmentState({ stepsMax: steps.length - 1 });
    }
  }, [setEnrollmentState, stepsMax]);

  useEffect(() => {
    if (sessionEnd) {
      const endDay = dayjs(sessionEnd);
      const diff = endDay.diff(dayjs(), 'millisecond');
      if (diff > 0) {
        const timeout = setTimeout(() => {
          reset();
          navigate(routes.timeout, { replace: true });
        });
        return () => {
          clearTimeout(timeout);
        };
      } else {
        navigate(routes.timeout, { replace: true });
      }
    } else {
      navigate(routes.timeout, { replace: true });
    }
  }, [sessionEnd, navigate, reset]);

  return (
    <PageContainer id="enrollment">
      <EnrollmentSideBar />
      <LogoContainer />
      {currentStep !== 4 && (
        <EnrollmentStepControls>
          <Button
            text={LL.common.controls.back()}
            size={controlsSize}
            styleVariant={ButtonStyleVariant.STANDARD}
            onClick={() => back()}
            disabled={steps[currentStep].backDisabled ?? false}
            icon={
              <ArrowSingle
                size={ArrowSingleSize.SMALL}
                direction={ArrowSingleDirection.LEFT}
              />
            }
          />
          <Button
            text={LL.common.controls.next()}
            size={controlsSize}
            styleVariant={ButtonStyleVariant.PRIMARY}
            onClick={() => nextSubject.next()}
            rightIcon={<ArrowSingle size={ArrowSingleSize.SMALL} />}
          />
        </EnrollmentStepControls>
      )}
      {steps[currentStep].step ?? null}
    </PageContainer>
  );
};

const steps: EnrollmentStep[] = [
  {
    key: 0,
    step: <WelcomeStep key={0} />,
    backDisabled: true,
  },
  {
    key: 1,
    step: <DataVerificationStep key={1} />,
  },
  {
    key: 2,
    step: <PasswordStep key={2} />,
  },
  {
    key: 3,
    step: <DeviceStep key={3} />,
    backDisabled: true,
  },
  {
    key: 4,
    step: <FinishStep key={4} />,
    backDisabled: true,
  },
];

type EnrollmentStep = {
  backDisabled?: boolean;
  key: string | number;
  step: ReactNode;
};
