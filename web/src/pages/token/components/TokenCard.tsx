import './style.scss';

import { zodResolver } from '@hookform/resolvers/zod';
import { useMemo } from 'react';
import { SubmitHandler, useForm } from 'react-hook-form';
import { useBreakpoint } from 'use-breakpoint';
import { z } from 'zod';

import { useI18nContext } from '../../../i18n/i18n-react';
import { FormInput } from '../../../shared/components/Form/FormInput/FormInput';
import { ArrowSingle } from '../../../shared/components/icons/ArrowSingle/ArrowSingle';
import {
  ArrowSingleDirection,
  ArrowSingleSize,
} from '../../../shared/components/icons/ArrowSingle/types';
import { Button } from '../../../shared/components/layout/Button/Button';
import {
  ButtonSize,
  ButtonStyleVariant,
} from '../../../shared/components/layout/Button/types';
import { Card } from '../../../shared/components/layout/Card/Card';
import { MessageBox } from '../../../shared/components/layout/MessageBox/MessageBox';
import { MessageBoxType } from '../../../shared/components/layout/MessageBox/types';
import { deviceBreakpoints } from '../../../shared/constants';

type FormFields = {
  token: string;
};

export const TokenCard = () => {
  const { breakpoint } = useBreakpoint(deviceBreakpoints);
  const { LL } = useI18nContext();
  const schema = useMemo(
    () =>
      z
        .object({
          token: z
            .string()
            .trim()
            .min(1, LL.pages.token.card.form.errors.token.required()),
        })
        .required(),
    [LL.pages.token.card.form.errors.token],
  );

  const { control, handleSubmit } = useForm<FormFields>({
    mode: 'all',
    defaultValues: {
      token: '',
    },
    resolver: zodResolver(schema),
  });

  const handleValidSubmit: SubmitHandler<FormFields> = (values) => {
    console.log(values);
  };

  return (
    <Card shaded={breakpoint !== 'mobile'} className="token-card">
      <h2>{LL.pages.token.card.title()}</h2>
      <MessageBox
        message={LL.pages.token.card.messageBox.email()}
        type={MessageBoxType.INFO}
        dismissId="token-page-card-email"
      />
      <form onSubmit={handleSubmit(handleValidSubmit)}>
        <FormInput
          controller={{ control, name: 'token' }}
          placeholder={LL.pages.token.card.form.fields.token.placeholder()}
          required
        />
        <div className="controls">
          <Button
            size={ButtonSize.LARGE}
            styleVariant={ButtonStyleVariant.PRIMARY}
            text={LL.pages.token.card.form.controls.submit()}
            rightIcon={
              <ArrowSingle
                direction={ArrowSingleDirection.RIGHT}
                size={ArrowSingleSize.LARGE}
              />
            }
          />
        </div>
      </form>
    </Card>
  );
};
