import { createFormHook } from '@tanstack/react-form';
import { FormInput } from './defguard-ui/components/form/FormInput/FormInput';
import { fieldContext, formContext } from './form-context';

export { useFieldContext, useFormContext } from './form-context';

export const { useAppForm, withForm } = createFormHook({
  fieldContext,
  formContext,
  fieldComponents: {
    FormInput,
  },
  formComponents: {},
});
