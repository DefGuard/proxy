import './style.scss';

import { ReactNode, useEffect } from 'react';
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
import { EnrollmentSideBar } from './components/EnrollmentSideBar/EnrollmentSideBar';
import { EnrollmentStepControls } from './components/EnrollmentStepControls/EnrollmentStepControls';
import { useEnrollmentStore } from './hooks/store/useEnrollmentStore';
import { DataVerificationStep } from './steps/DataVerificationStep/DataVerificationStep';
import { DeviceStep } from './steps/DeviceStep/DeviceStep';
import { FinishStep } from './steps/FinishStep/FinishStep';
import { PasswordStep } from './steps/PasswordStep/PasswordStep';
import { WelcomeStep } from './steps/WelcomeStep/WelcomeStep';

export const EnrollmentPage = () => {
  const { LL } = useI18nContext();
  const { breakpoint } = useBreakpoint(deviceBreakpoints);
  const currentStep = useEnrollmentStore((state) => state.step);
  const stepsMax = useEnrollmentStore((state) => state.stepsMax);
  const [setEnrollmentState, next, back] = useEnrollmentStore(
    (state) => [state.setState, state.nextStep, state.perviousStep],
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

  return (
    <PageContainer id="enrollment">
      <EnrollmentSideBar />
      <LogoContainer />
      <EnrollmentStepControls>
        <Button
          text={LL.common.controls.back()}
          size={controlsSize}
          styleVariant={ButtonStyleVariant.STANDARD}
          onClick={() => back()}
          disabled={currentStep === 0}
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
          onClick={() => next()}
          rightIcon={<ArrowSingle size={ArrowSingleSize.SMALL} />}
        />
      </EnrollmentStepControls>
      {steps[currentStep] ?? null}
    </PageContainer>
  );
};

const steps: ReactNode[] = [
  <WelcomeStep key={0} />,
  <DataVerificationStep key={1} />,
  <PasswordStep key={2} />,
  <DeviceStep key={3} />,
  <FinishStep key={4} />,
];
