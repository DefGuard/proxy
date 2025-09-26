import type { PropsWithChildren } from 'react';
import { Container } from '../Container/Container';
import './style.scss';
import clsx from 'clsx';
import { Icon } from '../../defguard-ui/components/Icon';
import type { IconKindValue } from '../../defguard-ui/components/Icon/icon-types';

export const ContainerWithIcon = ({
  children,
  className,
  id,
  icon,
}: PropsWithChildren & {
  icon: IconKindValue;
  className?: string;
  id?: string;
}) => {
  return (
    <Container id={id} className={clsx('container-with-icon', className)}>
      <div className="track">
        <div className="container-icon">
          <Icon icon={icon} />
        </div>
        <div className="content">{children}</div>
      </div>
    </Container>
  );
};
