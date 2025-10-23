import './style.scss';

import { useQuery } from '@tanstack/react-query';
import { useLoaderData } from '@tanstack/react-router';
import { capitalCase } from 'change-case';
import { QRCodeCanvas } from 'qrcode.react';
import { lazy, Suspense, useMemo, useState } from 'react';
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
import { IconButton } from '../../../shared/defguard-ui/components/IconButton/IconButton';
import { IconButtonMenu } from '../../../shared/defguard-ui/components/IconButtonMenu/IconButtonMenu';
import type { MenuItemsGroup } from '../../../shared/defguard-ui/components/Menu/types';
import { SizedBox } from '../../../shared/defguard-ui/components/SizedBox/SizedBox';
import { ThemeSpacing } from '../../../shared/defguard-ui/types';
import { getClientArtifactsQueryOptions } from '../../../shared/query/queryOptions';
import { openClientLink } from '../../../shared/utils/openVirtualLink';

const AppleHelpModal = lazy(async () => ({
  default: (await import('../../../shared/components/AppleHelpModal/AppleHelpModal'))
    .AppleHelpModal,
}));

export const ConfigureClientPage = () => {
  const pageData = useLoaderData({
    from: '/client-setup',
  });

  const { data: clientLinks } = useQuery(getClientArtifactsQueryOptions);

  const [appleHelpModalOpen, setAppleHelpModalOpen] = useState(false);

  const appleMenu = useMemo(
    (): MenuItemsGroup[] => [
      {
        header: {
          text: m.client_download_apple_help_header(),
          onHelp: () => setAppleHelpModalOpen(true),
        },
        items: [
          {
            icon: 'apple',
            text: 'Intel',
            onClick: () => openClientLink(clientLinks?.macos_amd64),
          },
          {
            icon: 'apple',
            text: 'ARM',
            onClick: () => openClientLink(clientLinks?.macos_arm64),
          },
        ],
      },
    ],
    [clientLinks],
  );

  const linuxMenu = useMemo(() => {
    const res: MenuItemsGroup[] = [
      {
        header: {
          text: `${capitalCase(m.misc_for())} Linux`,
        },
        items: [
          {
            icon: 'ubuntu',
            text: 'Ubuntu 24.04 ARM',
            onClick: () => openClientLink(clientLinks?.deb_arm64),
          },
          {
            icon: 'ubuntu',
            text: 'Ubuntu 24.04 AMD64',
            onClick: () => openClientLink(clientLinks?.deb_amd64),
          },
        ],
      },
      {
        items: [
          {
            icon: 'debian',
            text: 'Ubuntu 22.04 / Debian 12&13 ARM',
            onClick: () => openClientLink(clientLinks?.deb_legacy_arm64),
          },
          {
            icon: 'debian',
            text: 'Ubuntu 22.04 / Debian 12&13 AMD64',
            onClick: () => openClientLink(clientLinks?.deb_legacy_amd64),
          },
        ],
      },
      {
        items: [
          {
            icon: 'linux',
            text: 'RPM ARM',
            onClick: () => openClientLink(clientLinks?.rpm_arm64),
          },
          {
            icon: 'linux',
            text: 'RPM AMD64',
            onClick: () => openClientLink(clientLinks?.rpm_amd64),
          },
        ],
      },
      {
        items: [
          {
            icon: 'arch-linux',
            text: 'Arch Linux',
            onClick: () => openClientLink(externalLink.client.desktop.linux.arch),
          },
        ],
      },
    ];
    return res;
  }, [clientLinks]);

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
      <SizedBox height={ThemeSpacing.Xl3} />
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
          <div className="download">
            <p>{m.client_setup_download_label()}</p>
            <a
              href={clientLinks?.windows_amd64 ?? externalLink.defguard.download}
              target="_blank"
              rel="noopener noreferrer"
            >
              <IconButton icon="windows" />
            </a>
            <IconButtonMenu icon="apple" menuItems={appleMenu} />
            <IconButtonMenu icon="linux" menuItems={linuxMenu} />
          </div>
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
              intent: manualOpen ? m.controls_hide() : m.controls_show(),
            })}
          </span>
        </button>
      </ContainerWithIcon>
      {pageData.enrollmentData.user.enrolled && (
        <>
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
        </>
      )}
      <footer>
        <p className="finish">{m.client_setup_footer_extra()}</p>
        <ContactFooter email={pageData.enrollmentData.admin.email} />
      </footer>
      <Suspense fallback={null}>
        <AppleHelpModal
          isOpen={appleHelpModalOpen}
          onClose={() => {
            setAppleHelpModalOpen(false);
          }}
        />
      </Suspense>
    </Page>
  );
};
