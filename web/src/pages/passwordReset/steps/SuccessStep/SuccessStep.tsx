import './style.scss';

import { useNavigate } from 'react-router-dom';
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
import { MessageBoxOld } from '../../../../shared/components/layout/MessageBox/MessageBoxOld';
import { MessageBoxType } from '../../../../shared/components/layout/MessageBox/types';
import { routes } from '../../../../shared/routes';
import { usePasswordResetStore } from '../../hooks/usePasswordResetStore';

export const SuccessStep = () => {
  const navigate = useNavigate();
  const { LL } = useI18nContext();
  const [reset] = usePasswordResetStore((state) => [state.reset], shallow);

  return (
    <>
      <div className="controls single">
        <Button
          text={LL.pages.resetPassword.steps.resetSuccess.controls.back()}
          size={ButtonSize.LARGE}
          styleVariant={ButtonStyleVariant.LINK}
          icon={
            <ArrowSingle
              size={ArrowSingleSize.SMALL}
              direction={ArrowSingleDirection.LEFT}
            />
          }
          onClick={() => {
            reset();
            navigate(routes.main);
          }}
        />
      </div>
      <Card id="reset-succeded-card" data-testid="password-reset-success">
        <MessageBoxOld
          type={MessageBoxType.SUCCESS}
          message={LL.pages.resetPassword.steps.resetSuccess.messageBox()}
        />
      </Card>
    </>
  );
};
