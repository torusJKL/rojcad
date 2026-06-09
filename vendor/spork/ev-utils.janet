(defn nursery
  "Group a number of fibers into a single object for structured concurrency"
  []
  @{:supervisor (ev/chan) :fibers @{}})

(defn go-nursery
  "Spawn a fiber into a nursery, similar to ev/go."
  [nurse f &opt value]
  (def super (get nurse :supervisor))
  (def fibs (get nurse :fibers))
  (def fib (ev/go f value super))
  (set (fibs fib) fib))

(defmacro spawn-nursery
  "Similar to ev/spawn but associate spawned fibers with a nursery"
  [nurse & body]
  ~(,go-nursery ,nurse (fn _spawn [&] ,;body)))

(defn- drain-fibers
  "Canceling a group of fibers and wait for them all to complete."
  [super fibers reason]
  (each f fibers (ev/cancel f reason))
  (def n (length fibers))
  (table/clear fibers)
  (repeat n (ev/take super)))

(defn join-nursery
  "Suspend the current fiber until the nursery is emptied."
  [nurse]
  (def fibs (get nurse :fibers))
  (def super (get nurse :supervisor))
  (defer (drain-fibers super fibs "parent canceled")
    (while (next fibs)
      (def [sig fiber] (ev/take super))
      (if (= sig :ok)
        (put fibs fiber nil)
        (do
          (drain-fibers super fibs "sibling canceled")
          (propagate (fiber/last-value fiber) fiber))))))
