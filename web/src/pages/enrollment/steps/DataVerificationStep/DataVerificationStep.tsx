import { zodResolver } from '@hookform/resolvers/zod';
import { useMemo } from 'react';
import { SubmitHandler, useForm } from 'react-hook-form';
import { z } from 'zod';

import { useI18nContext } from '../../../../i18n/i18n-react';
import { FormInput } from '../../../../shared/components/Form/FormInput/FormInput';
import { Card } from '../../../../shared/components/layout/Card/Card';
import { MessageBox } from '../../../../shared/components/layout/MessageBox/MessageBox';
import { MessageBoxType } from '../../../../shared/components/layout/MessageBox/types';
import { EnrollmentStepIndicator } from '../../components/EnrollmentStepIndicator/EnrollmentStepIndicator';

type FormFields = {
  phone?: string;
};

export const DataVerificationStep = () => {
  const { LL } = useI18nContext();

  const pageLL = LL.pages.enrollment.steps.dataVerification;

  const schema = useMemo(
    () =>
      z.object({
        phone: z.string().trim().optional(),
      }),
    [],
  );

  const { control, handleSubmit } = useForm<FormFields>({
    defaultValues: {
      phone: '',
    },
    mode: 'all',
    resolver: zodResolver(schema),
  });

  const handleValidSubmit: SubmitHandler<FormFields> = (values) => {
    console.table(values);
  };

  return (
    <Card id="enrollment-data-verification-card">
      <EnrollmentStepIndicator />
      <h3>{pageLL.title()}</h3>
      <MessageBox type={MessageBoxType.INFO} message={pageLL.messageBox()} />
      <form onSubmit={handleSubmit(handleValidSubmit)}>
        <div className="row">
          <div className="item">
            <label>{pageLL.form.fields.firstName.label()}</label>
            <p>PLACEHOLDER</p>
          </div>
          <div className="item">
            <label>{pageLL.form.fields.lastName.label()}</label>
            <p>PLACEHOLDER</p>
          </div>
        </div>
        <div className="row">
          <div className="item">
            <label>{pageLL.form.fields.email.label()}</label>
            <p>PLACEHOLDER</p>
          </div>
          <FormInput
            label={pageLL.form.fields.phone.label()}
            controller={{ control, name: 'phone' }}
          />
        </div>
      </form>
    </Card>
  );
};
