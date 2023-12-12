import './style.scss';

import { zodResolver } from '@hookform/resolvers/zod';
import { useMutation } from '@tanstack/react-query';
import { AxiosError } from 'axios';
import { useMemo, useRef } from 'react';
import { SubmitHandler, useForm } from 'react-hook-form';
import { useNavigate } from 'react-router-dom';
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
import { routes } from '../../../../shared/routes';
import { usePasswordResetStore } from '../../hooks/usePasswordResetStore';

type FormFields = {
  email: string;
};

export const EmailStep = () => {
  const navigate = useNavigate();
  const { LL } = useI18nContext();
  const submitRef = useRef<HTMLInputElement | null>(null);

  const { passwordReset } = useApi();

  const schema = useMemo(
    () =>
      z.object({
        email: z
          .string()
          .trim()
          .min(1, LL.form.errors.required())
          .email(LL.form.errors.email()),
      }),
    [LL.form.errors],
  );

  const { control, handleSubmit } = useForm<FormFields>({
    mode: 'all',
    defaultValues: {
      email: '',
    },
    resolver: zodResolver(schema),
  });

  const setStore = usePasswordResetStore((state) => state.setState);

  const [next] = usePasswordResetStore((state) => [state.nextStep], shallow);

  const { mutate } = useMutation({
    mutationFn: passwordReset.request,
    onSuccess: () => {
      setStore({ loading: false });
      next(1);
    },
    onError: (err: AxiosError) => {
      setStore({ loading: false });
      console.error(err.message);
    },
  });

  const handleValidSubmit: SubmitHandler<FormFields> = (values) => {
    setStore({ loading: true });
    mutate(values);
  };

  return (
    <>
      <div className="controls">
        <Button
          size={ButtonSize.LARGE}
          styleVariant={ButtonStyleVariant.LINK}
          text={LL.common.controls.back()}
          icon={
            <ArrowSingle
              size={ArrowSingleSize.SMALL}
              direction={ArrowSingleDirection.LEFT}
            />
          }
          onClick={() => {
            next(0);
            navigate(routes.main);
          }}
        />
        <Button
          size={ButtonSize.LARGE}
          styleVariant={ButtonStyleVariant.PRIMARY}
          text={LL.pages.resetPassword.steps.email.controls.send()}
          rightIcon={
            <ArrowSingle
              size={ArrowSingleSize.SMALL}
              direction={ArrowSingleDirection.RIGHT}
            />
          }
          onClick={() => submitRef.current?.click()}
        />
      </div>
      <Card>
        <h2>{LL.pages.resetPassword.steps.email.title()}</h2>
        <form onSubmit={handleSubmit(handleValidSubmit)}>
          <FormInput
            controller={{ control, name: 'email' }}
            label={LL.pages.resetPassword.steps.email.form.fields.email.label()}
            required
          />
          <input className="hidden" type="submit" ref={submitRef} />
        </form>
      </Card>
    </>
  );
};
