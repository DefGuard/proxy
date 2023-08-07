import './style.scss';

import {
  autoUpdate,
  flip,
  FloatingPortal,
  offset,
  size,
  useFloating,
} from '@floating-ui/react';
import classNames from 'classnames';
import { AnimatePresence, motion } from 'framer-motion';
import { isUndefined, last } from 'lodash-es';
import { useEffect, useId, useMemo, useRef, useState } from 'react';
import { debounceTime, filter, Subject } from 'rxjs';
import { useBreakpoint } from 'use-breakpoint';

import { deviceBreakpoints } from '../../../constants';
import { detectClickInside } from '../../../utils/detectClickOutside';
import { isComparableWithStrictEquality } from '../../../utils/isComparable';
import { ArrowSingle } from '../../icons/ArrowSingle/ArrowSingle';
import { ArrowSingleDirection, ArrowSingleSize } from '../../icons/ArrowSingle/types';
import { useTheme } from '../hooks/theme/useTheme';
import { LoaderSpinner } from '../LoaderSpinner/LoaderSpinner';
import { Tag } from '../Tag/Tag';
import { SelectOptionRow } from './components/SelectOptionRow/SelectOptionRow';
import {
  SelectFloatingOption,
  SelectOption,
  SelectProps,
  SelectSizeVariant,
} from './types';

const compare = <T,>(v: T, other: T): boolean => {
  if (!isComparableWithStrictEquality(v)) {
    throw Error('Value is not comparable, make sure you provided identify method !');
  }

  return v === other;
};

export const Select = <T,>({
  onChangeArray,
  identify,
  onChangeSingle,
  onSearch,
  searchFilter,
  onRemove,
  onCreate,
  options,
  placeholder,
  selected,
  label,
  invalid,
  errorMessage,
  addOptionLabel,
  searchable = false,
  loading = false,
  disabled = false,
  searchDebounce = 500,
  searchMinLength = 1,
  disableLabelColon = false,
  inForm = false,
  disableOpen = false,
  sizeVariant = SelectSizeVariant.STANDARD,
  'data-testid': testId,
}: SelectProps<T>) => {
  const { breakpoint } = useBreakpoint(deviceBreakpoints);
  const { colors } = useTheme();
  const selectId = useId();
  const [open, setOpen] = useState(false);
  // used value for filtering options
  const [searchValue, setSearchValue] = useState<string | undefined>();
  // only for display
  const [searchDisplayValue, setSearchDisplayValue] = useState<string | undefined>();
  const searchRef = useRef<HTMLInputElement | null>(null);
  const searchPushRef = useRef<HTMLSpanElement | null>(null);
  const [searchFocused, setSearchFocused] = useState(false);
  const [searchSubject] = useState<Subject<string | undefined>>(new Subject());
  const extendable = useMemo(() => !isUndefined(onCreate), [onCreate]);

  const multi = Array.isArray(selected);

  const identifiers = useMemo(() => {
    if (Array.isArray(selected) && !isUndefined(identify)) {
      return selected.map((v) => identify(v));
    }
    return [];
  }, [identify, selected]);

  const { x, y, strategy, refs } = useFloating({
    open,
    onOpenChange: setOpen,
    placement: 'bottom',
    middleware: [
      offset(5),
      flip(),
      size({
        apply: ({ rects, elements }) => {
          Object.assign(elements.floating.style, {
            width: `${rects.reference.width}px`,
          });
        },
      }),
    ],
    whileElementsMounted: (refElement, floatingElement, updateFunc) =>
      autoUpdate(refElement, floatingElement, updateFunc),
  });

  const handleSelect = (value: T): void => {
    if (Array.isArray(selected)) {
      if (!onChangeArray) {
        throw Error('onChangeArray was not suplited when selected is an array');
      }
      if (selected.length) {
        if (isComparableWithStrictEquality(value) && !identify) {
          const includes = selected.includes(value);
          if (includes) {
            onChangeArray(selected.filter((v) => v !== value));
          } else {
            onChangeArray([...selected, value]);
          }
        } else {
          if (!identify) {
            throw Error('Select component needs identify method for comparing Objects');
          }
          const includes = selected
            .map((v) => identify(v))
            .find((v) => v === identify(value));
          if (includes) {
            onChangeArray(selected.filter((v) => identify(v) !== identify(value)));
          } else {
            onChangeArray([...selected, value]);
          }
        }
      } else {
        onChangeArray?.([value]);
      }
    } else {
      setOpen(false);
      if (!onChangeSingle) {
        throw Error(
          'onChangeSingle was not suplied when selected value was not an array',
        );
      }
      onChangeSingle(value);
    }
  };

  const getClassName = useMemo(() => {
    return classNames(
      'select-container',
      {
        disabled,
        loading,
        open,
        multi,
        selected: Array.isArray(selected)
          ? selected && selected.length
          : !isUndefined(selected),
        'in-form': inForm,
      },
      `size-${sizeVariant.valueOf().toLocaleLowerCase()}`,
    );
  }, [disabled, inForm, loading, multi, open, selected, sizeVariant]);

  const showSelectInnerPlaceholder = useMemo(() => {
    if (Array.isArray(selected)) {
      if (selected.length) {
        return false;
      }
    } else {
      if (selected) {
        return false;
      }
    }
    if (searchValue) {
      return false;
    }
    return true;
  }, [searchValue, selected]);

  const getSearchInputLength = useMemo(() => {
    const searchLength = searchValue?.length ?? 0;
    if (searchLength > 0) {
      return searchLength * 8;
    }
    if (placeholder) {
      return placeholder.length * 8 || 2;
    }
    return 2;
  }, [searchValue?.length, placeholder]);

  const renderTags = useMemo(() => {
    if (isUndefined(selected) && !Array.isArray(selected) && !multi) {
      return null;
    }
    if (Array.isArray(selected) && selected.length) {
      let selectedOptions: SelectOption<T>[] = [];

      if (isComparableWithStrictEquality(selected[0]) && !identify) {
        selectedOptions = selected.map(
          (v) => options.find((o) => o.value === v) as SelectOption<T>,
        );
      } else {
        if (identify) {
          selectedOptions = selected.map(
            (v) =>
              options.find((o) => identify(o.value) === identify(v)) as SelectOption<T>,
          );
        }
      }

      return selectedOptions.map((option) => (
        <Tag
          key={option.key}
          text={option.label}
          disposable
          onDispose={() => {
            onRemove?.(option.value);
          }}
        />
      ));
    }
    return null;
  }, [identify, multi, onRemove, options, selected]);

  const renderInner = useMemo(() => {
    if (searchFocused) return null;

    if (
      !searchable &&
      (!selected || (selected && Array.isArray(selected) && selected.length === 0))
    ) {
      return <motion.span className="placeholder">{placeholder}</motion.span>;
    }

    if (selected && !Array.isArray(selected) && !searchFocused) {
      const option = options.find((o) => o.value === selected);
      return (
        <motion.span className="selected-option">
          {option?.label ?? String(option?.value)}
        </motion.span>
      );
    }
  }, [options, placeholder, searchFocused, searchable, selected]);

  // options in float are only for presentation
  const floatingOptions = useMemo((): SelectFloatingOption<T>[] => {
    let availableOptions: SelectOption<T>[] = options;

    if (searchable && searchValue && searchValue.length) {
      if (!searchFilter) {
        throw Error('Select needs to be suplied with searchFilter when searchable');
      }
      availableOptions = searchFilter(searchValue, options);
    }

    if (isUndefined(selected)) {
      return availableOptions.map((o) => ({ ...o, selected: false }));
    }

    return availableOptions.map((o) => {
      if (Array.isArray(selected)) {
        const isComparable = isComparableWithStrictEquality(o.value);

        if (isComparable && !identify) {
          return { ...o, selected: selected.includes(o.value) };
        } else {
          if (!identify) {
            throw Error('Select component needs identify method for comparing Objects');
          }
          return {
            ...o,
            selected: identifiers?.includes(identify(o.value)) ?? false,
          };
        }
      } else {
        if (isComparableWithStrictEquality(o.value)) {
          return { ...o, selected: compare(o.value, selected) };
        } else {
          if (!identify) {
            throw Error(
              'Select needs to be suplied with identify method when values are objects',
            );
          }

          return { ...o, selected: identify(o.value) === identify(selected) };
        }
      }
    });
  }, [identifiers, identify, options, searchFilter, searchValue, searchable, selected]);

  const focusSearch = () => {
    if (searchable && searchRef && !searchFocused) {
      searchRef.current?.focus();
    }
  };

  // search sub
  useEffect(() => {
    const sub = searchSubject
      .pipe(
        debounceTime(searchDebounce),
        filter(
          (searchValue) =>
            !isUndefined(searchValue) && searchValue.length >= searchMinLength,
        ),
      )
      .subscribe((searchValue) => {
        if (onSearch) {
          onSearch(searchValue);
        }
        setSearchValue(searchValue);
      });
    return () => sub.unsubscribe();
  }, [onSearch, searchDebounce, searchMinLength, searchSubject]);

  // check click outside
  useEffect(() => {
    const clickHandler = (env: MouseEvent) => {
      const selectRect = refs.reference.current?.getBoundingClientRect();
      const floatingRect = refs.floating.current?.getBoundingClientRect();
      if (selectRect) {
        const rects = [selectRect as DOMRect];
        if (floatingRect) {
          rects.push(floatingRect);
        }
        const clickedInside = detectClickInside(env, rects);
        if (!clickedInside) {
          setOpen(false);
        }
      }
    };
    document.addEventListener('click', clickHandler);
    return () => {
      document.removeEventListener('click', clickHandler);
    };
  }, [refs.floating, refs.reference]);

  return (
    <div className="select" data-testid={testId}>
      {label && label.length > 0 && (
        <label className="select-label" htmlFor={selectId}>
          {label}
          {!disableLabelColon && ':'}
        </label>
      )}
      <motion.div
        ref={refs.setReference}
        id={selectId}
        className={getClassName}
        onClick={() => {
          if (open) {
            if (searchable) {
              focusSearch();
            }
          } else {
            if (!disabled && !loading && !disableOpen) {
              setOpen(true);
              if (searchable) {
                focusSearch();
              }
            }
          }
        }}
      >
        <div className="inner-frame">
          {renderInner}
          <div className="content-frame">
            {renderTags}
            {searchable && (
              <div className="search-frame">
                <span className="input-push" ref={searchPushRef}>
                  {!isUndefined(searchDisplayValue) &&
                    searchDisplayValue.length > 0 &&
                    searchDisplayValue.replace(' ', '&nbsp;')}
                  {showSelectInnerPlaceholder ? placeholder : null}
                </span>
                <input
                  type="text"
                  className="select-search"
                  value={searchValue}
                  onFocus={() => setSearchFocused(true)}
                  onBlur={() => setSearchFocused(false)}
                  onKeyDown={(event) => {
                    if (event.key === 'Enter') {
                      event.preventDefault();
                      event.stopPropagation();
                    }
                    if (multi) {
                      if (event.key === 'Backspace' && searchValue?.length === 0) {
                        if (Array.isArray(selected)) {
                          const lastSelected = last(selected);
                          if (lastSelected) {
                            handleSelect(lastSelected);
                          }
                        }
                      }
                    }
                  }}
                  onChange={(event) => {
                    const searchValue = event.target.value;
                    setSearchDisplayValue(searchValue);
                    searchSubject.next(searchValue);
                    if (!searchValue || searchValue.length === 0) {
                      // clear search / set to default list
                      if (onSearch) {
                        onSearch(undefined);
                      }
                    }
                  }}
                  ref={searchRef}
                  placeholder={showSelectInnerPlaceholder ? placeholder : undefined}
                  style={{
                    width: `${getSearchInputLength}px`,
                    color: searchFocused ? colors.textBodyPrimary : 'transparent',
                  }}
                />
              </div>
            )}
          </div>
          <div className="side">
            {loading ? (
              <LoaderSpinner size={22} />
            ) : (
              <ArrowSingle
                size={ArrowSingleSize.SMALL}
                direction={ArrowSingleDirection.DOWN}
              />
            )}
          </div>
        </div>
        <AnimatePresence mode="wait">
          {invalid && !open && errorMessage ? (
            <motion.span
              className="error-message"
              initial={{
                x: 20,
                opacity: 0,
                bottom: 0,
              }}
              animate={{
                x: 20,
                opacity: 1,
                bottom: -20,
              }}
              exit={{
                opacity: 0,
                x: 20,
                bottom: -20,
              }}
            >
              {errorMessage}
            </motion.span>
          ) : null}
        </AnimatePresence>
      </motion.div>
      <FloatingPortal>
        <AnimatePresence mode="wait">
          {open && options && (floatingOptions.length > 0 || extendable) && (
            <motion.div
              className="select-floating-ui"
              ref={refs.setFloating}
              style={{
                position: strategy,
                left: x || 0,
                top: y || 0,
              }}
              initial={{
                opacity: 0,
              }}
              animate={{
                opacity: 1,
              }}
              exit={{
                opacity: 0,
              }}
              transition={{
                duration: 0.1,
              }}
            >
              <div className="options-container">
                {extendable && breakpoint !== 'desktop' && (
                  <SelectOptionRow
                    label={addOptionLabel}
                    createOption
                    onClick={(e) => {
                      e.preventDefault();
                      e.stopPropagation();
                      onCreate?.();
                      setOpen(false);
                    }}
                  />
                )}
                {floatingOptions?.map((option) => {
                  return (
                    <SelectOptionRow
                      key={option.key}
                      label={option.label}
                      onClick={(e) => {
                        e.preventDefault();
                        e.stopPropagation();
                        handleSelect(option.value);
                      }}
                      selected={option.selected}
                    />
                  );
                })}
                {extendable && breakpoint === 'desktop' && (
                  <SelectOptionRow
                    label={addOptionLabel}
                    createOption
                    onClick={(e) => {
                      e.preventDefault();
                      e.stopPropagation();
                      onCreate?.();
                      setOpen(false);
                    }}
                  />
                )}
              </div>
            </motion.div>
          )}
        </AnimatePresence>
      </FloatingPortal>
    </div>
  );
};
