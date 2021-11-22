import * as React from 'react';
import { counter } from './counter';
import { mut as mutate } from './mut';

let x = counter()
mutate(x)

import('./dynamic').then(mod => mod.default())