import './style.scss';

import { shallow } from 'zustand/shallow';

import { useI18nContext } from '../../../../i18n/i18n-react';
import { ArrowSingle } from '../../../../shared/components/icons/ArrowSingle/ArrowSingle';
import {
  ArrowSingleDirection,
  ArrowSingleSize,
} from '../../../../shared/components/icons/ArrowSingle/types';
import { Button } from '../../../../shared/components/layout/Button/Button';
import {
  ButtonSize,
  ButtonStyleVariant,
} from '../../../../shared/components/layout/Button/types';
import { Card } from '../../../../shared/components/layout/Card/Card';
import { MessageBox } from '../../../../shared/components/layout/MessageBox/MessageBox';
import { MessageBoxType } from '../../../../shared/components/layout/MessageBox/types';
import { usePasswordResetStore } from '../../hooks/usePasswordResetStore';

export const FailureStep = () => {
  const { LL } = useI18nContext();
  const [reset] = usePasswordResetStore((state) => [state.reset], shallow);

  return (
    <>
      <div className="controls single">
        <Button
          text={LL.pages.resetPassword.steps.resetFailed.controls.back()}
          size={ButtonSize.LARGE}
          styleVariant={ButtonStyleVariant.LINK}
          icon={
            <ArrowSingle
              size={ArrowSingleSize.SMALL}
              direction={ArrowSingleDirection.LEFT}
            />
          }
          onClick={reset}
        />
      </div>
      <Card id="reset-failed-card">
        <MessageBox
          type={MessageBoxType.ERROR}
          message={LL.pages.resetPassword.steps.resetFailed.messageBox()}
        />
      </Card>
    </>
  );
};
