import { zodResolver } from '@hookform/resolvers/zod';
import { useMutation } from '@tanstack/react-query';
import type { AxiosError } from 'axios';
import { useMemo, useRef } from 'react';
import { type SubmitHandler, useForm } from 'react-hook-form';
import { z } from 'zod';
import { shallow } from 'zustand/shallow';

import { useI18nContext } from '../../../../i18n/i18n-react';
import { FormInput } from '../../../../shared/components/Form/FormInput/FormInput';
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
import { useApi } from '../../../../shared/hooks/api/useApi';
import { passwordValidator } from '../../../../shared/validators/password';
import { usePasswordResetStore } from '../../hooks/usePasswordResetStore';

type FormFields = {
  password: string;
  repeat: string;
};

export const PasswordStep = () => {
  const { LL } = useI18nContext();
  const submitRef = useRef<HTMLInputElement | null>(null);

  const {
    passwordReset: { reset },
  } = useApi();

  const schema = useMemo(
    () =>
      z
        .object({
          password: passwordValidator(LL),
          repeat: z.string().trim().nonempty(LL.form.errors.required()),
        })
        .refine((data) => data.password === data.repeat, {
          message: LL.pages.resetPassword.steps.resetPassword.form.errors.repeat(),
          path: ['repeat'],
        }),
    [LL],
  );

  const setStore = usePasswordResetStore((state) => state.setState);

  const [next] = usePasswordResetStore((state) => [state.nextStep], shallow);

  const { mutate } = useMutation({
    mutationFn: reset,
    onSuccess: () => {
      setStore({ loading: false });
      next(3);
    },
    onError: (err: AxiosError) => {
      setStore({ loading: false });
      console.error(err.message);
    },
  });

  const { control, handleSubmit } = useForm<FormFields>({
    defaultValues: {
      password: '',
      repeat: '',
    },
    mode: 'all',
    resolver: zodResolver(schema),
  });

  const handleValidSubmit: SubmitHandler<FormFields> = (values) => {
    setStore({ loading: true });
    mutate(values);
  };

  return (
    <>
      <div className="controls single">
        <Button
          data-testid="password-reset-submit"
          type="button"
          size={ButtonSize.LARGE}
          styleVariant={ButtonStyleVariant.PRIMARY}
          rightIcon={
            <ArrowSingle
              size={ArrowSingleSize.SMALL}
              direction={ArrowSingleDirection.RIGHT}
            />
          }
          text={LL.pages.resetPassword.steps.resetPassword.controls.submit()}
          onClick={() => submitRef.current?.click()}
        />
      </div>
      <Card id="reset-password-card">
        <h2>{LL.pages.resetPassword.steps.resetPassword.title()}</h2>
        <form onSubmit={handleSubmit(handleValidSubmit)}>
          <FormInput
            type="password"
            controller={{ control, name: 'password' }}
            label={LL.pages.resetPassword.steps.resetPassword.form.fields.password.label()}
          />
          <FormInput
            type="password"
            controller={{ control, name: 'repeat' }}
            label={LL.pages.resetPassword.steps.resetPassword.form.fields.repeat.label()}
          />
          <input type="submit" ref={submitRef} className="hidden" />
        </form>
      </Card>
    </>
  );
};
