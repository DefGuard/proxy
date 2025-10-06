import './style.scss';

import { useLoaderData } from '@tanstack/react-router';
import { QRCodeCanvas } from 'qrcode.react';
import { useState } from 'react';
import { m } from '../../../paraglide/messages';
import { ContactFooter } from '../../../shared/components/ContactFooter/ContactFooter';
import { ContainerWithIcon } from '../../../shared/components/ContainerWithIcon/ContainerWithIcon';
import { Page } from '../../../shared/components/Page/Page';
import { EnrollmentStep } from '../../../shared/components/Step/Step';
import { externalLink } from '../../../shared/consts';
import { Button } from '../../../shared/defguard-ui/components/Button/Button';
import { CopyField } from '../../../shared/defguard-ui/components/CopyField/CopyField';
import { Divider } from '../../../shared/defguard-ui/components/Divider/Divider';
import { Fold } from '../../../shared/defguard-ui/components/Fold/Fold';
import { Icon } from '../../../shared/defguard-ui/components/Icon';
import { SizedBox } from '../../../shared/defguard-ui/components/SizedBox/SizedBox';
import { ThemeSpacing } from '../../../shared/defguard-ui/types';

export const ConfigureClientPage = () => {
  const pageData = useLoaderData({
    from: '/client-setup',
  });
  const [manualOpen, setManualOpen] = useState(false);

  const deepLink = () =>
    `defguard://addinstance?token=${pageData.token}&url=${pageData.enrollmentData.instance.proxy_url}`;

  const qrData = () => {
    return btoa(
      JSON.stringify({
        url: pageData.enrollmentData.instance.proxy_url,
        token: pageData.token,
      }),
    );
  };

  return (
    <Page id="configure-client-page">
      <EnrollmentStep current={2} max={2} />
      <header>
        <h1>{m.client_setup_title()}</h1>
        <p>{m.client_setup_subtitle()}</p>
      </header>
      <SizedBox height={ThemeSpacing.Xl5} />
      <ContainerWithIcon icon="desktop" id="setup-desktop">
        <header>
          <h5>{m.client_setup_desktop_title()}</h5>
          <p>{m.client_setup_desktop_auto_title()}</p>
          <p>
            {m.client_setup_desktop_auto_explain_1()}{' '}
            <span>{m.client_setup_desktop_auto_explain_2()}</span>
          </p>
        </header>
        <div className="buttons">
          <a href={deepLink()} target="_blank">
            <Button
              text={m.client_setup_desktop_auto_button_one_click()}
              variant="primary"
              iconRight="open-in-new-window"
            />
          </a>
        </div>
        <Divider orientation="horizontal" />
        <Fold open={manualOpen} contentClassName="manual">
          <p className="title">{m.client_setup_desktop_manual_title()}</p>
          <p className="subtitle">{m.client_setup_desktop_manual_subtitle()}</p>
          <SizedBox height={ThemeSpacing.Xl2} />
          <div className="form-col-2">
            <CopyField
              text={pageData.enrollmentData.instance.proxy_url}
              copyTooltip={m.cmp_copy_field_tooltip()}
              label={m.form_label_url()}
            />
            <CopyField
              copyTooltip={m.cmp_copy_field_tooltip()}
              text={pageData.token}
              label={m.form_label_token()}
            />
          </div>
          <SizedBox height={ThemeSpacing.Xl2} />
        </Fold>
        <button
          className="fold-button"
          onClick={() => {
            setManualOpen((s) => !s);
          }}
        >
          <Icon icon="config" />
          <span>
            {m.client_setup_desktop_manual_fold({
              intent: m.controls_show(),
            })}
          </span>
        </button>
      </ContainerWithIcon>
      <SizedBox height={ThemeSpacing.Md} />
      <ContainerWithIcon id="setup-mobile" icon="phone">
        <header>
          <h5>{m.client_setup_mobile_title()}</h5>
          <p>{m.client_setup_mobile_subtitle()}</p>
        </header>
        <div className="bottom">
          <div className="qr">
            <QRCodeCanvas value={qrData()} size={100} />
          </div>
          <div className="download">
            <p>{m.client_setup_mobile_forgot()}</p>
            <div className="links">
              <a
                href={externalLink.client.mobile.google}
                target="_blank"
                rel="noopener noreferrer"
              >
                <Button
                  variant="outlined"
                  iconLeft="android"
                  text={m.client_setup_mobile_google()}
                />
              </a>
              <a
                href={externalLink.client.mobile.apple}
                target="_blank"
                rel="noopener noreferrer"
              >
                <Button
                  variant="outlined"
                  iconLeft="apple"
                  text={m.client_setup_mobile_apple()}
                />
              </a>
            </div>
          </div>
        </div>
      </ContainerWithIcon>
      <footer>
        <p className="finish">{m.client_setup_footer_extra()}</p>
        <ContactFooter email={pageData.enrollmentData.admin.email} />
      </footer>
    </Page>
  );
};
