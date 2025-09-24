import { createFormHook, createFormHookContexts } from '@tanstack/react-form';
import { FormInput } from './components/form/FormInput/FormInput';

export const { fieldContext, formContext, useFieldContext, useFormContext } =
  createFormHookContexts();

export const { useAppForm, withFieldGroup, withForm } = createFormHook({
  fieldContext,
  formContext,
  fieldComponents: {
    FormInput,
  },
  formComponents: {},
});
