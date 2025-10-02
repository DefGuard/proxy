import { useLoaderData, useNavigate } from '@tanstack/react-router';
import { m } from '../../../paraglide/messages';
import { ContainerWithIcon } from '../../../shared/components/ContainerWithIcon/ContainerWithIcon';
import { Page } from '../../../shared/components/Page/Page';
import { PageNavigation } from '../../../shared/components/PageNavigation/PageNavigation';
import { EnrollmentStep } from '../../../shared/components/Step/Step';
import { Divider } from '../../../shared/defguard-ui/components/Divider/Divider';
import './style.scss';
import { revalidateLogic } from '@tanstack/react-form';
import type { AxiosError } from 'axios';
import z from 'zod';
import { api } from '../../../shared/api/api';
import { Button } from '../../../shared/defguard-ui/components/Button/Button';
import { SizedBox } from '../../../shared/defguard-ui/components/SizedBox/SizedBox';
import { useAppForm } from '../../../shared/defguard-ui/form';
import { ThemeSpacing } from '../../../shared/defguard-ui/types';
import { isPresent } from '../../../shared/defguard-ui/utils/isPresent';
import { useEnrollmentStore } from '../../../shared/hooks/useEnrollmentStore';

const formSchema = z.object({
  token: z.string().trim().min(1, m.form_error_required()),
});

type FormFields = z.infer<typeof formSchema>;

const defaultValues: FormFields = {
  token: '',
};

export const EnrollmentStartPage = () => {
  const navigate = useNavigate();
  const loaderData = useLoaderData({
    from: '/enrollment-start',
  });
  const setEnrollment = useEnrollmentStore((s) => s.setState);

  const form = useAppForm({
    defaultValues,
    validationLogic: revalidateLogic({
      mode: 'change',
      modeAfterSubmission: 'change',
    }),
    validators: {
      onDynamic: formSchema,
      onSubmit: formSchema,
    },
    onSubmit: async ({ value, formApi }) => {
      const response = await api.enrollment.start
        .callbackFn({
          data: {
            token: value.token,
          },
        })
        .catch((e: AxiosError) => {
          if (e.status === 401) {
            formApi.setErrorMap({
              onSubmit: {
                fields: {
                  token: m.form_error_token(),
                },
              },
            });
          }
        });

      if (!response) {
        return;
      }

      setEnrollment({
        enrollmentData: response.data,
        token: value.token,
      });

      navigate({
        to: '/client-setup',
        replace: true,
      });
    },
  });

  return (
    <Page id="enrollment-start-page" nav>
      <EnrollmentStep current={1} max={2} />
      <header>
        <h1>{m.enrollment_start_title()}</h1>
        <p>{m.enrollment_start_subtitle()}</p>
      </header>
      <SizedBox height={ThemeSpacing.Xl5} />
      {isPresent(loaderData?.url) && isPresent(loaderData.button_display_name) && (
        <>
          <ContainerWithIcon icon="globe">
            <header>
              <h5>{m.enrollment_start_external_title()}</h5>
              <SizedBox height={ThemeSpacing.Xs} />
              <p>{m.enrollment_start_external_subtitle()}</p>
            </header>
            <div className="openid-link">
              <a href={loaderData.url} target="_blank">
                <Button
                  size="big"
                  iconRight="open-in-new-window"
                  text={m.cmp_openid_button({
                    provider: loaderData.button_display_name,
                  })}
                />
              </a>
            </div>
          </ContainerWithIcon>
          <Divider text={m.misc_or()} orientation="horizontal" />
        </>
      )}
      <ContainerWithIcon icon="file">
        <header>
          <h5>{m.enrollment_start_internal_title()}</h5>
          <SizedBox height={ThemeSpacing.Xs} />
          <p>{m.enrollment_start_internal_subtitle()}</p>
        </header>
        <form.AppForm>
          <form
            onSubmit={(e) => {
              e.preventDefault();
              e.stopPropagation();
              form.handleSubmit();
            }}
          >
            <form.AppField name="token">
              {(field) => <field.FormInput required label={m.form_label_token()} />}
            </form.AppField>
          </form>
        </form.AppForm>
      </ContainerWithIcon>
      <PageNavigation
        backText={m.controls_back()}
        onBack={() => {
          navigate({
            to: '/',
          });
        }}
        nextText={m.controls_continue()}
        onNext={() => {
          form.handleSubmit();
        }}
      />
    </Page>
  );
};
