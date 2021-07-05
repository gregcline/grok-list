(ns gregcline.grok-list-ui.events
    (:require
     [re-frame.core :as re-frame]
     [gregcline.grok-list-ui.db :as db]
     ))

(re-frame/reg-event-db
 ::initialize-db
 (fn [_ _]
   db/default-db))
