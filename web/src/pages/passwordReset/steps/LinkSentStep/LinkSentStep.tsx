import './style.scss';

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

export const LinkSentStep = () => {
  const { LL } = useI18nContext();
  return (
    <>
      <div className="controls single">
        <Button
          text={LL.pages.resetPassword.steps.linkSent.controls.back()}
          size={ButtonSize.LARGE}
          styleVariant={ButtonStyleVariant.LINK}
          icon={
            <ArrowSingle
              size={ArrowSingleSize.SMALL}
              direction={ArrowSingleDirection.LEFT}
            />
          }
        />
      </div>
      <Card id="link-sent-card">
        <MessageBox
          type={MessageBoxType.INFO}
          message={LL.pages.resetPassword.steps.linkSent.messageBox()}
        />
      </Card>
    </>
  );
};
