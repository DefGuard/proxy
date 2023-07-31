import './style.scss';

import { zodResolver } from '@hookform/resolvers/zod';
import { useMemo } from 'react';
import { SubmitHandler, useForm } from 'react-hook-form';
import { z } from 'zod';

import { useI18nContext } from '../../../../i18n/i18n-react';
import { FormInput } from '../../../../shared/components/Form/FormInput/FormInput';
import { Card } from '../../../../shared/components/layout/Card/Card';
import { passwordValidator } from '../../../../shared/validators/password';
import { EnrollmentStepIndicator } from '../../components/EnrollmentStepIndicator/EnrollmentStepIndicator';

type FormFields = {
  password: string;
  repeat: string;
};

export const PasswordStep = () => {
  const { LL } = useI18nContext();

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

  const handleValidSubmit: SubmitHandler<FormFields> = (values) => {
    console.table(values);
  };

  return (
    <Card id="enrollment-password-card">
      <EnrollmentStepIndicator />
      <h3>{pageLL.title()}</h3>
      <form onSubmit={handleSubmit(handleValidSubmit)}>
        <FormInput
          label={pageLL.form.fields.password.label()}
          controller={{ control, name: 'password' }}
        />
        <FormInput
          label={pageLL.form.fields.repeat.label()}
          controller={{ control, name: 'repeat' }}
        />
      </form>
    </Card>
  );
};
