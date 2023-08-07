import './style.scss';

import { zodResolver } from '@hookform/resolvers/zod';
import { useMutation } from '@tanstack/react-query';
import { AxiosError } from 'axios';
import { useEffect, useMemo, useRef } from 'react';
import { SubmitHandler, useForm } from 'react-hook-form';
import { z } from 'zod';
import { shallow } from 'zustand/shallow';

import { useI18nContext } from '../../../../i18n/i18n-react';
import { FormInput } from '../../../../shared/components/Form/FormInput/FormInput';
import { Card } from '../../../../shared/components/layout/Card/Card';
import { useApi } from '../../../../shared/hooks/api/useApi';
import { passwordValidator } from '../../../../shared/validators/password';
import { EnrollmentStepIndicator } from '../../components/EnrollmentStepIndicator/EnrollmentStepIndicator';
import { useEnrollmentStore } from '../../hooks/store/useEnrollmentStore';

type FormFields = {
  password: string;
  repeat: string;
};

export const PasswordStep = () => {
  const {
    enrollment: { activateUser },
  } = useApi();
  const submitRef = useRef<HTMLInputElement | null>(null);
  const { LL } = useI18nContext();

  const userInfo = useEnrollmentStore((state) => state.userInfo);
  const [nextSubject, next] = useEnrollmentStore(
    (state) => [state.nextSubject, state.nextStep],
    shallow,
  );

  const pageLL = LL.pages.enrollment.steps.password;

  const schema = useMemo(
    () =>
      z
        .object({
          password: passwordValidator(LL),
          repeat: z.string().nonempty(LL.form.errors.required()),
        })
        .refine((values) => values.password === values.repeat, {
          message: pageLL.form.fields.repeat.errors.matching(),
          path: ['repeat'],
        }),
    [LL, pageLL.form.fields.repeat.errors],
  );

  const { handleSubmit, control } = useForm<FormFields>({
    defaultValues: {
      password: '',
      repeat: '',
    },
    mode: 'all',
    criteriaMode: 'all',
    resolver: zodResolver(schema),
  });

  const { mutate, isLoading } = useMutation({
    mutationFn: activateUser,
    onSuccess: () => {
      next();
    },
    onError: (err: AxiosError) => {
      console.error(err.message);
    },
  });

  const handleValidSubmit: SubmitHandler<FormFields> = (values) => {
    if (!isLoading && userInfo && userInfo.phone) {
      mutate({
        phone_number: userInfo.phone,
        password: values.password,
      });
    }
  };

  useEffect(() => {
    const sub = nextSubject.subscribe(() => {
      submitRef.current?.click();
    });

    return () => {
      sub.unsubscribe();
    };
  }, [nextSubject, submitRef]);

  return (
    <Card id="enrollment-password-card">
      <EnrollmentStepIndicator />
      <h3>{pageLL.title()}</h3>
      <form onSubmit={handleSubmit(handleValidSubmit)}>
        <FormInput
          label={pageLL.form.fields.password.label()}
          controller={{ control, name: 'password' }}
          type="password"
          floatingErrors={{
            title: LL.form.errors.password.floatingTitle(),
          }}
          autoComplete="new-password"
        />
        <FormInput
          label={pageLL.form.fields.repeat.label()}
          controller={{ control, name: 'repeat' }}
          type="password"
          autoComplete="new-password"
        />
        <input className="hidden" type="submit" ref={submitRef} />
      </form>
    </Card>
  );
};
