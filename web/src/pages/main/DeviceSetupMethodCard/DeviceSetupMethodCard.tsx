import './style.scss';

import type { ReactNode } from 'react';

import { Button } from '../../../shared/components/layout/Button/Button';
import {
  ButtonSize,
  ButtonStyleVariant,
} from '../../../shared/components/layout/Button/types';

type Props = {
  title: string;
  subtitle: string;
  logo: ReactNode;
  onSelect: () => void;
  testId?: string;
};
export const PathSelectCard = ({ title, logo, subtitle, onSelect, testId }: Props) => {
  return (
    <div className="device-setup-method">
      <h3>{title}</h3>
      <p className="sub-title">{subtitle}</p>
      {logo && <div className="logo-wrapper">{logo}</div>}
      <Button
        size={ButtonSize.LARGE}
        text="Select"
        styleVariant={ButtonStyleVariant.PRIMARY}
        onClick={onSelect}
        data-testid={testId}
      />
    </div>
  );
};
