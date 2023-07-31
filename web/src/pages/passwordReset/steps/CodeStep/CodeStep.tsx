import './style.scss';

import { zodResolver } from '@hookform/resolvers/zod';
import { useMemo, useRef } from 'react';
import { SubmitHandler, useForm } from 'react-hook-form';
import { z } from 'zod';

import { useI18nContext } from '../../../../i18n/i18n-react';
import { FormInput } from '../../../../shared/components/Form/FormInput/FormInput';
import { ArrowSingle } from '../../../../shared/components/icons/ArrowSingle/ArrowSingle';
import { ArrowSingleSize } from '../../../../shared/components/icons/ArrowSingle/types';
import { Button } from '../../../../shared/components/layout/Button/Button';
import {
  ButtonSize,
  ButtonStyleVariant,
} from '../../../../shared/components/layout/Button/types';
import { Card } from '../../../../shared/components/layout/Card/Card';
import { MessageBox } from '../../../../shared/components/layout/MessageBox/MessageBox';
import { MessageBoxType } from '../../../../shared/components/layout/MessageBox/types';

type FormFields = {
  code: string;
};

export const CodeStep = () => {
  const { LL } = useI18nContext();
  const submitRef = useRef<HTMLInputElement | null>(null);

  const schema = useMemo(
    () =>
      z.object({
        code: z.string().trim().min(1, LL.form.errors.required()),
      }),
    [LL.form.errors],
  );

  const { control, handleSubmit } = useForm<FormFields>({
    defaultValues: {
      code: '',
    },
    resolver: zodResolver(schema),
    mode: 'all',
  });

  const handleValidSubmit: SubmitHandler<FormFields> = (values) => {
    console.table(values);
  };

  return (
    <>
      <div className="controls">
        <Button
          size={ButtonSize.LARGE}
          styleVariant={ButtonStyleVariant.SAVE}
          text={LL.pages.resetPassword.steps.securityCode.controls.sendCode()}
        />
        <Button
          size={ButtonSize.LARGE}
          rightIcon={<ArrowSingle size={ArrowSingleSize.SMALL} />}
          styleVariant={ButtonStyleVariant.PRIMARY}
          text={LL.common.controls.next()}
          onClick={() => submitRef.current?.click()}
        />
      </div>
      <Card id="security-code-card">
        <h2>{LL.pages.resetPassword.steps.securityCode.title()}</h2>
        <MessageBox
          dismissId="reset-password-security-code"
          type={MessageBoxType.INFO}
          message={LL.pages.resetPassword.steps.securityCode.messagebox()}
        />
        <form onSubmit={handleSubmit(handleValidSubmit)}>
          <FormInput
            controller={{ control, name: 'code' }}
            label={LL.pages.resetPassword.steps.securityCode.form.fields.code.label()}
          />
          <input type="submit" ref={submitRef} className="hidden" />
        </form>
      </Card>
    </>
  );
};
