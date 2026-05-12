# IRQ-safe VGA logging

Ce document explique pourquoi l'affichage VGA peut poser problème quand il est
utilisé depuis du code normal **et** depuis des handlers d'interruptions, puis
explique la solution actuellement utilisée dans le projet.

---

## 1. C'est quoi une IRQ ?

Une **IRQ** (*Interrupt Request*) est une interruption matérielle.

Exemples classiques sur un PC i386 :

- le timer périodique,
- le clavier,
- d'autres périphériques.

Quand une IRQ arrive :

1. le CPU interrompt le code en cours,
2. il sauve l'état nécessaire,
3. il saute dans un handler d'interruption,
4. puis il revient au code interrompu.

Donc une IRQ peut arriver **au milieu** d'une fonction normale.

---

## 2. Pourquoi c'est un problème pour le logger VGA ?

Le logger VGA est protégé par un `Mutex`.

L'idée du `Mutex` est simple :

> une seule exécution à la fois a le droit de modifier la console.

Le problème arrive quand une IRQ tente de logger pendant que le code normal
est déjà en train de logger.

### Scénario du deadlock

Imaginons :

1. du code normal appelle `print!`,
2. `print!` prend le lock du logger,
3. **avant de finir**, une IRQ arrive,
4. le handler IRQ appelle aussi `print!`,
5. lui aussi essaie de prendre le même lock,
6. il boucle en attendant le lock,
7. mais le lock est détenu par le code interrompu,
8. or ce code interrompu ne peut pas reprendre tant que l'IRQ n'est pas finie.

Résultat : **deadlock**.

---

## 3. Pourquoi un `Mutex` seul ne suffit pas

Un `spin::Mutex` protège bien contre deux accès concurrents, mais il ne sait pas
que l'un des deux accès peut venir d'une interruption sur le **même CPU**.

Donc :

- le `Mutex` protège la donnée,
- mais il ne bloque pas l'arrivée d'une IRQ locale pendant que le lock est tenu.

---

## 4. L'idée de la solution

La solution classique en kernel est :

1. désactiver les interruptions maskables localement,
2. prendre le lock,
3. faire le travail,
4. relâcher le lock,
5. restaurer l'état précédent des interruptions.

Sur x86 i386, cela passe typiquement par :

- `cli` : désactive les interruptions maskables,
- `sti` : réactive les interruptions maskables.

Le point important est qu'on ne veut **pas** faire juste :

```text
cli
...
sti
```

dans tous les cas.

On veut d'abord savoir si les interruptions étaient déjà désactivées avant
d'entrer dans la section critique.

Pourquoi ?

Parce que si on est déjà dans un contexte où elles étaient coupées, il serait
faux de les réactiver à la sortie.

---

## 5. Le code utilisé dans le projet

### 5.1 `without_interrupts`

Code actuel :

```rust
pub fn without_interrupts<R>(f: impl FnOnce() -> R) -> R
{
    let were_enabled = interrupts_enabled();

    if were_enabled {
        unsafe {
            cli();
        }
    }

    let result = f();

    if were_enabled {
        unsafe {
            sti();
        }
    }

    result
}
```

### Lecture ligne par ligne

#### `pub fn without_interrupts<R>(...) -> R`

- `pub fn` : fonction publique du module.
- `<R>` : `R` est un type générique pour la valeur de retour.
- ça veut dire que la fonction marche quelle que soit la valeur renvoyée par
  la closure.

Exemple :

- si la closure renvoie `()`, alors `R = ()`,
- si elle renvoie `bool`, alors `R = bool`,
- si elle renvoie `u32`, alors `R = u32`.

#### `f: impl FnOnce() -> R`

Ici, `f` est une **closure** (ou toute fonction équivalente) qui :

- ne prend pas d'argument,
- renvoie une valeur de type `R`.

`FnOnce` veut dire :

> on garantit seulement qu'on peut l'appeler au moins une fois.

C'est le bon choix ici, car la closure peut capturer des valeurs par mouvement.

#### `let were_enabled = interrupts_enabled();`

On lit l'état actuel du flag d'interruption (`IF` dans `EFLAGS`).

On mémorise donc :

- `true` si les interruptions étaient activées,
- `false` sinon.

#### `if were_enabled { cli(); }`

Si les interruptions étaient activées, on les désactive.

Si elles étaient déjà désactivées, on ne touche à rien.

#### `let result = f();`

On exécute la closure dans cette zone protégée.

#### `if were_enabled { sti(); }`

On ne réactive les interruptions **que** si elles étaient activées à l'entrée.

Donc on restaure l'état précédent au lieu d'imposer un nouvel état arbitraire.

#### `result`

La fonction retourne ce qu'a renvoyé la closure.

---

### 5.2 `with_logger`

Code actuel :

```rust
fn with_logger<R>(f: impl FnOnce(&mut VgaConsole) -> R) -> R
{
    cpu::without_interrupts(|| {
        let mut logger = logger().lock();
        f(&mut logger)
    })
}
```

Tu avais demandé une explication détaillée de cette partie, donc voici.

---

## 6. Décomposition de `with_logger`

### Signature

```rust
fn with_logger<R>(f: impl FnOnce(&mut VgaConsole) -> R) -> R
```

### Qu'est-ce que ça veut dire ?

Cette fonction prend une closure `f` qui :

- reçoit un `&mut VgaConsole`,
- fait quelque chose avec,
- renvoie éventuellement une valeur `R`.

En français :

> "donne-moi une opération à faire sur la console VGA, et je vais l'exécuter
> de façon sûre, avec le logger verrouillé et les IRQ locales coupées".

---

## 7. Lecture ligne par ligne de `with_logger`

### Ligne 1

```rust
cpu::without_interrupts(|| {
```

On appelle `without_interrupts` en lui donnant une closure.

Cette closure sera exécutée :

- avec interruptions maskables locales désactivées si elles étaient actives,
- puis l'état sera restauré à la sortie.

Le `||` signifie :

> "closure sans argument".

---

### Ligne 2

```rust
let mut logger = logger().lock();
```

On fait deux choses :

1. `logger()` récupère la référence globale vers le `Mutex<VgaConsole>`.
2. `.lock()` prend le lock du mutex.

Le résultat n'est pas directement un `VgaConsole`, mais un **guard** de mutex.

Ce guard :

- garantit que le lock est détenu,
- donne accès au `VgaConsole`,
- libère automatiquement le lock quand il sort de portée.

Le `mut` est nécessaire parce qu'on va muter la console à travers ce guard.

---

### Ligne 3

```rust
f(&mut logger)
```

Ici, on appelle la closure fournie par l'appelant.

`logger` est un guard de mutex, mais grâce à `DerefMut`, on peut obtenir un
`&mut VgaConsole` via `&mut logger`.

Donc cette ligne veut dire :

> "appelle l'opération demandée sur la console VGA protégée".

La valeur de retour de `f(...)` devient aussi la valeur de retour de
`with_logger(...)`.

---

## 8. `_print` ligne par ligne

Code actuel :

```rust
#[doc(hidden)]
pub(crate) fn _print(args: fmt::Arguments)
{
    with_logger(|logger| {
        fmt::write(logger, args).ok();
    });
}
```

### `args: fmt::Arguments`

`print!` et `println!` ne construisent pas directement une `String` ici.

Ils produisent un objet de type `fmt::Arguments` qui décrit :

- le texte,
- les valeurs formatées,
- la façon de les écrire.

---

### `with_logger(|logger| { ... })`

On dit :

> "exécute cette closure sur la console VGA, en mode IRQ-safe et avec le mutex pris".

Le `|logger|` signifie :

> "cette closure reçoit un argument nommé `logger`".

Ici, `logger` est un `&mut VgaConsole`.

---

### `fmt::write(logger, args).ok();`

`fmt::write(...)` écrit le contenu de `args` dans `logger`.

Ça marche parce que `VgaConsole` implémente `core::fmt::Write`.

Le `.ok()` transforme le `Result` en `Option` et ignore l'erreur éventuelle.

En pratique ici, ça veut surtout dire :

> "essaie d'écrire, et ne fais rien de spécial si l'écriture renvoie une erreur".

---

## 9. Vue d'ensemble de `_print`

Quand `_print(args)` est appelé, le flux logique est :

1. on entre dans `with_logger(...)`,
2. `with_logger` appelle `without_interrupts(...)`,
3. les IRQ locales sont temporairement coupées si besoin,
4. on prend le `Mutex<VgaConsole>`,
5. `fmt::write(...)` écrit dans la console,
6. la closure se termine,
7. le guard est détruit, donc le lock est relâché,
8. les IRQ sont restaurées si elles étaient actives avant.

---

## 10. Ce que cette solution protège réellement

Cette solution protège contre :

- la réentrance par **IRQ maskable locale** sur le même CPU,
- le deadlock classique "code normal + IRQ + même lock".

---

## 11. Ce que cette solution **ne** protège **pas**

### 1. Les NMI

`cli` ne bloque pas les NMI.

### 2. Toutes les exceptions CPU

Certaines exceptions peuvent toujours arriver.

### 3. Le SMP complet à lui seul

Sur une machine multi-CPU :

- `without_interrupts` coupe seulement les IRQ du **CPU courant**,
- pas celles des autres CPU.

Le `Mutex` reste donc nécessaire pour l'exclusion mutuelle inter-CPU.

En pratique, sur SMP, le pattern kernel classique est :

- spinlock,
- IRQ locales désactivées pendant que le spinlock est tenu.

---

## 12. Pourquoi `with_logger` est plus clair qu'un appel direct partout

On aurait pu écrire partout quelque chose comme :

```rust
cpu::without_interrupts(|| {
    let mut logger = logger().lock();
    fmt::write(&mut *logger, args).ok();
});
```

Mais ce serait :

- répétitif,
- facile à oublier,
- plus dur à garder cohérent.

`with_logger(...)` permet donc de centraliser :

- la discipline IRQ-safe,
- la prise du lock,
- l'accès mutable à `VgaConsole`.

---

## 13. Modèle mental simple

Tu peux retenir :

### `without_interrupts(...)`

> "exécute ce bloc sans te faire interrompre par une IRQ maskable locale".

### `logger().lock()`

> "je prends l'accès exclusif à la console VGA".

### `with_logger(...)`

> "donne-moi une opération sur `VgaConsole`, je la fais avec le bon lock et
> avec les IRQ locales neutralisées pendant la section critique".

### `_print(...)`

> "utilise `with_logger(...)` pour écrire des arguments formatés dans la
> console VGA".

---

## 14. Résumé final

Le problème n'était pas "le VGA est concurrent".

Le vrai problème était :

> une interruption peut arriver pendant qu'on tient déjà le lock du logger,
> puis essayer de reprendre ce même lock.

La correction consiste donc à :

1. couper les IRQ locales,
2. prendre le lock,
3. écrire,
4. relâcher le lock,
5. restaurer l'état précédent des IRQ.

Et c'est exactement ce que fait la combinaison :

- `cpu::without_interrupts(...)`
- `logger().lock()`
- `with_logger(...)`
