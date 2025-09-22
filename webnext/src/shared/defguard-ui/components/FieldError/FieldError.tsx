import { AnimatePresence, motion } from 'motion/react';
import './style.scss';
import { motionTransitionStandard } from '../../../consts';
import { isPresent } from '../../utils/isPresent';

type Props = {
  error?: string;
};

export const FieldError = ({ error }: Props) => {
  return (
    <AnimatePresence mode="wait">
      {isPresent(error) && error.length > 0 && (
        <motion.p
          className="field-error"
          transition={motionTransitionStandard}
          initial={{
            x: -20,
            opacity: 0,
          }}
          animate={{
            x: 0,
            opacity: 1,
          }}
          exit={{
            x: -20,
            opacity: 0,
          }}
        >
          {error}
        </motion.p>
      )}
    </AnimatePresence>
  );
};
