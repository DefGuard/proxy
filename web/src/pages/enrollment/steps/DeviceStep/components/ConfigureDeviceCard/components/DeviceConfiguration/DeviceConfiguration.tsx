import './style.scss';

import { isUndefined } from 'lodash-es';
import { useEffect, useMemo, useState } from 'react';
import QRCode from 'react-qr-code';

import { useI18nContext } from '../../../../../../../../i18n/i18n-react';
import { ActionButton } from '../../../../../../../../shared/components/layout/ActionButton/ActionButton';
import { ActionButtonVariant } from '../../../../../../../../shared/components/layout/ActionButton/types';
import { Card } from '../../../../../../../../shared/components/layout/Card/Card';
import { useTheme } from '../../../../../../../../shared/components/layout/hooks/theme/useTheme';
import { Input } from '../../../../../../../../shared/components/layout/Input/Input';
import { MessageBox } from '../../../../../../../../shared/components/layout/MessageBox/MessageBox';
import { Select } from '../../../../../../../../shared/components/layout/Select/Select';
import { SelectOption } from '../../../../../../../../shared/components/layout/Select/types';
import SvgIconHamburger from '../../../../../../../../shared/components/svg/IconHamburger';
import { DeviceConfig } from '../../../../../../../../shared/hooks/api/types';
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

  useEffect(() => {
    if (deviceState?.configs && deviceState.configs.length) {
      setSelected(deviceState.configs[0]);
    }
  }, [deviceState?.configs]);

  return (
    <>
      <MessageBox
        message={autoMode ? cardLL.messageBox.auto() : cardLL.messageBox.manual()}
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
      <p className="qr-info">{cardLL.cardTitle()}</p>
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
                  navigator.clipboard
                    .writeText(selected.config)
                    .catch((e) => console.error(e));
                }
              }}
            />
            <ActionButton
              disabled={isUndefined(selected)}
              variant={ActionButtonVariant.DOWNLOAD}
              onClick={() => {
                if (selected) {
                  downloadWGConfig(
                    selected.config,
                    `${selected.network_name.toLowerCase().replace(' ', '-')}`,
                  );
                }
              }}
            />
          </div>
        </div>
        <div className="qr">
          {selected && (
            <QRCode
              size={275}
              value={selected.config}
              bgColor={colors.surfaceDefaultModal}
              fgColor={colors.textBodyPrimary}
            />
          )}
        </div>
      </Card>
    </>
  );
};
