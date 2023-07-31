import './style.scss';

import { useBreakpoint } from 'use-breakpoint';

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
import { Card } from '../../shared/components/layout/Card/Card';
import { PageContainer } from '../../shared/components/layout/PageContainer/PageContainer';
import { deviceBreakpoints } from '../../shared/constants';

export const SessionTimeoutPage = () => {
  const { breakpoint } = useBreakpoint(deviceBreakpoints);
  const { LL } = useI18nContext();
  return (
    <PageContainer id="session-timeout">
      <LogoContainer />
      <Card shaded={breakpoint === 'desktop'}>
        <h2>{LL.pages.sessionTimeout.card.header()}</h2>
        <p>{LL.pages.sessionTimeout.card.message()}</p>
      </Card>
      <div className="controls">
        <Button
          size={ButtonSize.LARGE}
          styleVariant={ButtonStyleVariant.LINK}
          icon={
            <ArrowSingle
              size={ArrowSingleSize.LARGE}
              direction={ArrowSingleDirection.LEFT}
            />
          }
          text={LL.pages.sessionTimeout.controls.back()}
        />
        <Button
          size={ButtonSize.LARGE}
          styleVariant={ButtonStyleVariant.PRIMARY}
          text={LL.pages.sessionTimeout.controls.contact()}
        />
      </div>
    </PageContainer>
  );
};
