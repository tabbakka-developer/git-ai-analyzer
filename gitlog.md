COMMIT: remove todo comment from UserController (2026-06-26)
M	Modules/Agent/Http/Controllers/UserController.php

COMMIT: restrict trip deletion for carriers based on organization settings | use full return logic for orders with inactive trips | fix layout in carrier edit view (2026-06-26)
M	Modules/Carrier/Http/Controllers/Route/IndexController.php
M	Modules/Consolidator/Resources/views/org/carrier/edit.blade.php
M	core/Facades/OrderFacade.php
M	workbench/datatable-constructor/src/Resources/views/scripts.blade.php

COMMIT: permission fixes | view updates (2026-06-26)
M	Modules/Agent/Facades/Controller/User/UserModel.php
M	Modules/Agent/Http/Controllers/UserController.php
M	Modules/Api/Http/Controllers/OrderController.php
M	Modules/Carrier/Entities/Route.php
M	Modules/Carrier/Http/Controllers/Route/IndexController.php
M	Modules/Consolidator/Facades/Controller/Report/Ticket/VgaRegistryReport.php
M	Modules/Consolidator/Facades/Controller/RouteCombinedModel.php
M	Modules/Consolidator/Http/Controllers/RouteCombined/IndexController.php
M	Modules/Consolidator/Resources/views/org/agent/edit.blade.php
M	Modules/Consolidator/Resources/views/org/carrier/edit.blade.php

COMMIT: added order_id (2026-06-23)
M	Modules/Api/Http/Controllers/TicketController.php

COMMIT: some fixes (2026-06-23)
M	Modules/Api/Http/routes.php
M	core/Jobs/SendTripChangedWebhookJob.php

COMMIT: finish api and webhook | update view for email settings | fix IDOR (2026-06-20)
M	Modules/Agent/Facades/Controller/User/UserModel.php
M	Modules/Backend/Resources/views/org/agent/edit.blade.php
M	core/Facades/Controller/Trip/IndexModel.php
M	core/Facades/TicketFacade.php

COMMIT: added webhook and api for uklon integration | bugfixing (2026-06-19)
M	Modules/Api/Http/Controllers/TicketController.php
M	Modules/Api/Http/routes.php
M	Modules/Backend/Resources/views/org/agent/edit.blade.php
M	core/Entities/Organization/Agent.php
A	core/Jobs/SendTripChangedWebhookJob.php
M	core/Observers/Trip/TripObserverTrait.php
