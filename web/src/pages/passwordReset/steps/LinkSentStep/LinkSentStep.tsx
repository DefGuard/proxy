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

export const LinkSentStep = () => {
  const { LL } = useI18nContext();
  const navigate = useNavigate();
  const [next] = usePasswordResetStore((state) => [state.nextStep], shallow);

  return (
    <>
      <div className="controls single">
        <Button
          text={LL.pages.resetPassword.steps.linkSent.controls.back()}
          size={ButtonSize.LARGE}
          styleVariant={ButtonStyleVariant.LINK}
          onClick={() => {
            navigate(routes.main);
            next(0);
          }}
          icon={
            <ArrowSingle
              size={ArrowSingleSize.SMALL}
              direction={ArrowSingleDirection.LEFT}
            />
          }
        />
      </div>
      <Card id="link-sent-card" data-testid="email-sent-message">
        <MessageBoxOld
          type={MessageBoxType.INFO}
          message={LL.pages.resetPassword.steps.linkSent.messageBox()}
        />
      </Card>
    </>
  );
};
