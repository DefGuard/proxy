import './style.scss';

import parse from 'html-react-parser';
import { isUndefined } from 'lodash-es';
import { useEffect, useMemo, useState } from 'react';
import QRCode from 'react-qr-code';

import { useI18nContext } from '../../../../../../../../i18n/i18n-react';
import { ActionButton } from '../../../../../../../../shared/components/layout/ActionButton/ActionButton';
import { ActionButtonVariant } from '../../../../../../../../shared/components/layout/ActionButton/types';
import { Card } from '../../../../../../../../shared/components/layout/Card/Card';
import { useTheme } from '../../../../../../../../shared/components/layout/hooks/theme/useTheme';
import { Input } from '../../../../../../../../shared/components/layout/Input/Input';
import { MessageBoxOld } from '../../../../../../../../shared/components/layout/MessageBox/MessageBoxOld';
import { MessageBoxType } from '../../../../../../../../shared/components/layout/MessageBox/types';
import { Select } from '../../../../../../../../shared/components/layout/Select/Select';
import type { SelectOption } from '../../../../../../../../shared/components/layout/Select/types';
import SvgIconHamburger from '../../../../../../../../shared/components/svg/IconHamburger';
import type { DeviceConfig } from '../../../../../../../../shared/hooks/api/types';
import { downloadWGConfig } from '../../../../../../../../shared/utils/downloadWGConfig';
import { useEnrollmentStore } from '../../../../../../hooks/store/useEnrollmentStore';

const networkIdentifier = (c: DeviceConfig): number => c.network_id;

export const DeviceConfiguration = () => {
  const { colors } = useTheme();
  const [selected, setSelected] = useState<DeviceConfig | undefined>();

  const { LL } = useI18nContext();

  const cardLL = LL.pages.enrollment.steps.deviceSetup.cards.device.config;

  const deviceState = useEnrollmentStore((state) => state.deviceState);

  const autoMode = !isUndefined(deviceState?.device?.privateKey);

  const selectOptions = useMemo(
    (): SelectOption<DeviceConfig>[] =>
      deviceState?.configs?.map((c) => ({
        value: c,
        label: c.network_name,
        key: c.network_id,
      })) ?? [],
    [deviceState?.configs],
  );

  const networksAvailable =
    deviceState && Array.isArray(deviceState.configs) && deviceState.configs.length > 0;

  const preparedConfig = useMemo(() => {
    if (deviceState?.device?.privateKey) {
      return selected?.config.replace('YOUR_PRIVATE_KEY', deviceState.device.privateKey);
    }

    if (deviceState?.device?.pubkey) {
      return selected?.config.replace('YOUR_PRIVATE_KEY', deviceState.device.pubkey);
    }

    return selected?.config;
  }, [selected, deviceState?.device?.privateKey, deviceState?.device?.pubkey]);

  useEffect(() => {
    if (deviceState?.configs?.length) {
      setSelected(deviceState.configs[0]);
    }
  }, [deviceState?.configs]);

  return (
    <>
      <MessageBoxOld
        message={parse(autoMode ? cardLL.messageBox.auto() : cardLL.messageBox.manual())}
      />
      <Input
        value={deviceState?.device?.name}
        label={cardLL.deviceNameLabel()}
        disabled
        onChange={(e) => {
          e.preventDefault();
          e.stopPropagation();
          return;
        }}
      />
      {networksAvailable && (
        <>
          <div className="qr-info">
            <p>{cardLL.cardTitle()}</p>
          </div>

          <Card id="device-config-card">
            <div className="top">
              <SvgIconHamburger />
              <label>{cardLL.card.selectLabel()}:</label>
              <Select<DeviceConfig>
                identify={networkIdentifier}
                options={selectOptions}
                onChangeSingle={(config) => setSelected(config)}
                selected={selected}
              />
              <div className="actions">
                <ActionButton variant={ActionButtonVariant.QRCODE} active />
                <ActionButton
                  variant={ActionButtonVariant.COPY}
                  disabled={isUndefined(selected) || !window.isSecureContext}
                  onClick={() => {
                    if (selected && window.isSecureContext) {
                      if (deviceState?.device?.privateKey && preparedConfig) {
                        navigator.clipboard
                          .writeText(preparedConfig)
                          .catch((e) => console.error(e));
                      } else {
                        navigator.clipboard
                          .writeText(selected.config)
                          .catch((e) => console.error(e));
                      }
                    }
                  }}
                />
                <ActionButton
                  disabled={isUndefined(selected)}
                  variant={ActionButtonVariant.DOWNLOAD}
                  onClick={() => {
                    if (preparedConfig && selected) {
                      downloadWGConfig(
                        deviceState?.device?.privateKey
                          ? preparedConfig
                          : selected.config,
                        `${selected.network_name.toLowerCase().replace(/\s+/g, '-')}`,
                      );
                    }
                  }}
                />
              </div>
            </div>
            <div className="qr">
              {!isUndefined(preparedConfig) && (
                <QRCode
                  size={275}
                  value={preparedConfig}
                  bgColor={colors.surfaceDefaultModal}
                  fgColor={colors.textBodyPrimary}
                />
              )}
            </div>
          </Card>
        </>
      )}
      {!networksAvailable && (
        <MessageBoxOld
          message={cardLL.noNetworksMessage()}
          type={MessageBoxType.WARNING}
        />
      )}
    </>
  );
};
