import clsx from 'clsx';
import { isPresent } from '../../../utils/isPresent';
import { InteractionBox } from '../../InteractionBox/InteractionBox';
import type { MenuHeaderProps } from '../types';

export const MenuHeader = ({ text, onHelp }: MenuHeaderProps) => {
  return (
    <div
      className={clsx('menu-header', {
        'with-help': isPresent(onHelp),
      })}
    >
      <p className="group-title">{text}</p>
      {isPresent(onHelp) && (
        <InteractionBox
          className="menu-header-help"
          icon="help"
          iconSize={20}
          onClick={onHelp}
        />
      )}
    </div>
  );
};
