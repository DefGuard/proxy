import './style.scss';
import { useNavigate } from '@tanstack/react-router';
import { useMemo, useState } from 'react';
import { m } from '../../paraglide/messages';
import { Page } from '../../shared/components/Page/Page';
import { PageNavigation } from '../../shared/components/PageNavigation/PageNavigation';
import { EnrollmentStep } from '../../shared/components/Step/Step';
import { externalLink } from '../../shared/consts';
import { Button } from '../../shared/defguard-ui/components/Button/Button';
import { ButtonMenu } from '../../shared/defguard-ui/components/ButtonMenu/MenuButton';
import { Icon } from '../../shared/defguard-ui/components/Icon';
import type { IconKindValue } from '../../shared/defguard-ui/components/Icon/icon-types';
import type { MenuItemsGroup } from '../../shared/defguard-ui/components/Menu/types';
import { Modal } from '../../shared/defguard-ui/components/Modal/Modal';
import { ModalControls } from '../../shared/defguard-ui/components/ModalControls/ModalControls';
import { SizedBox } from '../../shared/defguard-ui/components/SizedBox/SizedBox';
import { ThemeSpacing } from '../../shared/defguard-ui/types';
import { isPresent } from '../../shared/defguard-ui/utils/isPresent';
import androidIcon from './assets/android.png';
import iosIcon from './assets/ios.png';
import laptopIcon from './assets/laptop.png';
import desktopIcon from './assets/pc-tower.png';

// open link in onClick handler
const openLink = (value: string): void => {
  const anchorElement = document.createElement('a');
  anchorElement.style.display = 'none';
  anchorElement.href = value;
  anchorElement.target = '_blank';
  anchorElement.rel = 'noopener noreferrer';
  document.body.appendChild(anchorElement);
  anchorElement.click();
  anchorElement.remove();
};

const linuxMenu: MenuItemsGroup[] = [
  {
    items: [
      {
        text: 'Deb X86',
        onClick: () => openLink(externalLink.client.desktop.linux.deb.amd),
      },
      {
        text: 'Deb ARM',
        onClick: () => openLink(externalLink.client.desktop.linux.deb.arm),
      },
      {
        text: 'RPM X86',
        onClick: () => openLink(externalLink.client.desktop.linux.rpm.amd),
      },
      {
        text: 'RPM ARM',
        onClick: () => openLink(externalLink.client.desktop.linux.rpm.arm),
      },
      {
        text: 'ArchLinux',
        onClick: () => openLink(externalLink.client.desktop.linux.arch),
      },
    ],
  },
];

export const ClientDownloadPage = () => {
  const navigate = useNavigate();

  const [confirmModalOpen, setConfirmModalOpen] = useState(false);
  const [_appleHelpModalOpen, setAppleHelpModalOpen] = useState(false);

  const appleMenu = useMemo(
    (): MenuItemsGroup[] => [
      {
        header: {
          text: 'Apple',
          onHelp: () => {
            setAppleHelpModalOpen(true);
          },
        },
        items: [
          {
            text: 'Intel',
            onClick: () => openLink(externalLink.client.desktop.macos.intel),
          },
          {
            text: 'ARM',
            onClick: () => openLink(externalLink.client.desktop.macos.arm),
          },
        ],
      },
    ],
    [],
  );

  return (
    <Page id="client-download-page" nav>
      <EnrollmentStep current={0} max={2} />
      <header>
        <h1>{m.client_download_title()}</h1>
        <p>{m.client_download_subtitle()}</p>
      </header>
      <SizedBox height={ThemeSpacing.Xl5} />
      <div className="platforms">
        <div className="label">
          <Icon icon="desktop" size={20} /> <p>{m.client_download_label_desktop()}</p>
        </div>
        <Platform
          testId="windows"
          title={m.client_download_for({ platform: 'Windows' })}
          subtitle={m.client_download_supports_and({
            platform: 'Windows 10',
            other: 'Windows 11',
          })}
          buttonText={m.client_download_for({ platform: 'Windows' })}
          buttonIconKind="windows"
          directLink={externalLink.client.desktop.windows}
          icon={desktopIcon}
        />
        <Platform
          testId="linux"
          title={m.client_download_for({ platform: 'Linux' })}
          subtitle={m.client_download_supports_newer({
            platform: 'TODO',
          })}
          buttonText={m.client_download_for({ platform: 'Linux' })}
          buttonIconKind="linux"
          menuItems={linuxMenu}
          icon={desktopIcon}
        />
        <Platform
          testId="macos"
          title={m.client_download_for({ platform: 'Windows' })}
          subtitle={m.client_download_supports_newer({
            platform: 'macOS 14 (Sonoma)',
          })}
          buttonText={m.client_download_for({ platform: 'Mac' })}
          buttonIconKind="app-store"
          menuItems={appleMenu}
          icon={laptopIcon}
        />
      </div>
      <SizedBox height={ThemeSpacing.Xl3} />
      <div className="platforms">
        <div className="label">
          <Icon icon="mobile" size={20} /> <p>{m.client_download_label_mobile()}</p>
        </div>
        <Platform
          testId="android"
          title={m.client_download_for({ platform: 'Android' })}
          subtitle={m.client_download_supports_newer({
            platform: 'Android 12.0 (Snow Cone)',
          })}
          buttonText={m.client_download_for({ platform: 'Android' })}
          buttonIconKind="android"
          directLink={externalLink.client.mobile.google}
          icon={androidIcon}
        />
        <Platform
          testId="iOS"
          title={m.client_download_for({ platform: 'iOS' })}
          subtitle={m.client_download_supports_newer({
            platform: 'iOS 15+',
          })}
          buttonText={m.client_download_for({ platform: 'iOS' })}
          buttonIconKind="apple"
          directLink={externalLink.client.mobile.apple}
          icon={iosIcon}
        />
      </div>
      <Modal
        title={m.client_download_modal_title()}
        size="small"
        isOpen={confirmModalOpen}
        onClose={() => {
          setConfirmModalOpen(false);
        }}
      >
        <p>{m.client_download_modal_content()}</p>
        <ModalControls
          cancelProps={{
            text: m.client_download_modal_cancel(),
            onClick: () => setConfirmModalOpen(false),
          }}
          submitProps={{
            text: m.controls_continue(),
            onClick: () => {
              navigate({
                to: '/enrollment-start',
                replace: true,
              });
            },
          }}
        />
      </Modal>
      <PageNavigation
        backText={m.controls_back()}
        onBack={() => {
          navigate({
            to: '/',
            replace: true,
          });
        }}
        nextText={m.controls_continue()}
        onNext={() => {
          setConfirmModalOpen(true);
        }}
      />
    </Page>
  );
};

const Platform = ({
  buttonIconKind,
  buttonText,
  icon,
  subtitle,
  title,
  testId,
  menuItems,
  directLink,
}: {
  icon: string;
  title: string;
  subtitle: string;
  buttonText: string;
  buttonIconKind: IconKindValue;
  testId: string;
  directLink?: string;
  menuItems?: MenuItemsGroup[];
}) => {
  return (
    <div className="platform" data-testid={testId}>
      <img src={icon} loading="lazy" width={60} height={60} />
      <div className="description">
        <p>{title}</p>
        <p>{subtitle}</p>
      </div>
      {isPresent(directLink) && (
        <a href={directLink} target="_blank">
          <Button variant="outlined" iconLeft={buttonIconKind} text={buttonText} />
        </a>
      )}
      {isPresent(menuItems) && (
        <ButtonMenu
          variant="outlined"
          menuItems={menuItems}
          iconLeft={buttonIconKind}
          text={buttonText}
        />
      )}
    </div>
  );
};
