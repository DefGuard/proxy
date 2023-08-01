import { zodResolver } from '@hookform/resolvers/zod';
import { useMemo } from 'react';
import { SubmitHandler, useController, useForm } from 'react-hook-form';
import { z } from 'zod';

import { useI18nContext } from '../../../../../../../i18n/i18n-react';
import { FormInput } from '../../../../../../../shared/components/Form/FormInput/FormInput';
import { FormToggle } from '../../../../../../../shared/components/Form/FormToggle/FormToggle';
import { MessageBox } from '../../../../../../../shared/components/layout/MessageBox/MessageBox';
import { ToggleOption } from '../../../../../../../shared/components/layout/Toggle/types';

enum ConfigurationType {
  AUTO,
  MANUAL,
}

type FormFields = {
  name: string;
  configType: ConfigurationType;
  public?: string;
};

export const CreateDevice = () => {
  const { LL } = useI18nContext();

  const cardLL = LL.pages.enrollment.steps.deviceSetup.cards.device;

  const toggleOptions: ToggleOption<ConfigurationType>[] = useMemo(
    () => [
      {
        text: cardLL.create.form.fields.toggle.generate(),
        value: ConfigurationType.AUTO,
      },
      {
        text: cardLL.create.form.fields.toggle.own(),
        value: ConfigurationType.MANUAL,
      },
    ],
    [cardLL.create.form.fields.toggle],
  );

  const schema = useMemo(
    () =>
      z
        .object({
          name: z.string().trim().nonempty(LL.form.errors.required()),
          configType: z.number(),
          public: z.string().trim().optional(),
        })
        .superRefine((val, ctx) => {
          if (val.configType === ConfigurationType.MANUAL && val.public?.length === 0) {
            ctx.addIssue({
              code: z.ZodIssueCode.custom,
              message: LL.form.errors.required(),
              path: ['public'],
            });
          }
        }),
    [LL.form.errors],
  );

  const { control, handleSubmit } = useForm<FormFields>({
    defaultValues: {
      name: '',
      configType: ConfigurationType.AUTO,
      public: '',
    },
    resolver: zodResolver(schema),
    mode: 'all',
  });

  const {
    field: { value: configTypeValue },
  } = useController({ control, name: 'configType' });

  const handleValidSubmit: SubmitHandler<FormFields> = (values) => {
    console.table(values);
  };

  return (
    <>
      <MessageBox message={cardLL.create.messageBox()} />
      <form onSubmit={handleSubmit(handleValidSubmit)}>
        <FormInput
          label={cardLL.create.form.fields.name.label()}
          controller={{ control, name: 'name' }}
          required
        />
        <FormToggle
          controller={{ control, name: 'configType' }}
          options={toggleOptions}
        />
        <FormInput
          label={cardLL.create.form.fields.public.label()}
          controller={{ control, name: 'public' }}
          disabled={configTypeValue === ConfigurationType.AUTO}
        />
      </form>
    </>
  );
};
