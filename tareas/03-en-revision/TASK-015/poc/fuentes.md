# Fuentes de F1-13 (TASK-015)

Documentación oficial de GitHub y GitLab consultada en solo lectura (GET de
la página, sin sesión) el 2026-11-14. Por cada URL: fecha de consulta, el
tier que la página declara para lo que cita el ADR y una cita de menos de 15
palabras. `poc-f1-13.py` lee esta tabla: cada URL de la tabla de
equivalencias del ADR tiene que estar aquí, y cada celda de tier del ADR
tiene que coincidir con el tier de una de las URL que cita esa fila.

Convenciones del tier:

- GitHub: «Free público; Pro/Team/Enterprise» = repos públicos con Free y
  públicos y privados con Pro, Team y Enterprise Cloud; «org pública;
  Enterprise Cloud» = repos públicos de organización o privados de
  organización con Enterprise Cloud; «todos» = la página no pone ninguna
  restricción de plan (no se ha probado en un privado de pago).
- GitLab: el «Tier» que la página pone en su cabecera o en la sección
  citada (Free, Premium, Ultimate), para GitLab.com.
- «precios»: página de planes; se cita para los nombres y límites de los
  planes, no para una garantía concreta.

<!-- fuentes:inicio -->
| id | URL | consultada | tier | cita |
|---|---|---|---|---|
| D01 | https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/about-rulesets | 2026-11-14 | Free público; Pro/Team/Enterprise | «Rulesets are available in public repositories with GitHub Free» |
| D02 | https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/available-rules-for-rulesets | 2026-11-14 | Free público; Pro/Team/Enterprise | «Dismiss stale pull request approvals when new commits are pushed» |
| D03 | https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/configuring-pull-request-merges/managing-a-merge-queue | 2026-11-14 | org pública; Enterprise Cloud | «available in any public repository owned by an organization» |
| D04 | https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/customizing-your-repository/about-code-owners | 2026-11-14 | Free público; Pro/Team/Enterprise | «define code owners in public repositories with GitHub Free» |
| D05 | https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-protected-branches/about-protected-branches | 2026-11-14 | Free público; Pro/Team/Enterprise | «Protected branches are available in public repositories with GitHub Free» |
| D06 | https://docs.github.com/en/apps/creating-github-apps/about-creating-github-apps/deciding-when-to-build-a-github-app | 2026-11-14 | todos | «GitHub Apps can act independently of a user» |
| D07 | https://docs.github.com/en/rest/checks/runs | 2026-11-14 | todos | «Lists check runs for a commit ref» |
| D08 | https://docs.github.com/en/authentication/managing-commit-signature-verification/about-commit-signature-verification | 2026-11-14 | todos | «GitHub marks the commit or tag "Verified"» |
| D09 | https://docs.github.com/en/rest/pulls/reviews | 2026-11-14 | todos | «The review actions include: APPROVE, REQUEST_CHANGES, or COMMENT» |
| D10 | https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/enabling-features-for-your-repository/managing-github-actions-settings-for-a-repository | 2026-11-14 | todos | «workflows are not allowed to create or approve pull requests» |
| D11 | https://docs.github.com/en/rest/pulls/pulls | 2026-11-14 | todos | «Responses include a maximum of 3000 files» |
| D12 | https://docs.github.com/en/actions/how-tos/write-workflows/choose-what-workflows-do/use-secrets | 2026-11-14 | todos | «not passed to the runner when a workflow is triggered from a forked repository» |
| D13 | https://docs.github.com/en/actions/reference/security/secure-use | 2026-11-14 | todos | «self-hosted runners should almost never be used for public repositories» |
| D14 | https://docs.github.com/en/actions/how-tos/manage-runners/self-hosted-runners/apply-labels | 2026-11-14 | todos | «Assigning a label to a self-hosted runner» |
| D15 | https://github.com/pricing | 2026-11-14 | precios | «Free Public repositories Team Included Enterprise Included» |
| L01 | https://docs.gitlab.com/user/project/repository/branches/protected/ | 2026-11-14 | Free | «You can only delete protected branches with the GitLab UI or API.» |
| L02 | https://docs.gitlab.com/user/project/merge_requests/approvals/ | 2026-11-14 | Premium | «These approvals are optional and don’t prevent merging without approval.» |
| L03 | https://docs.gitlab.com/user/project/merge_requests/approvals/settings/ | 2026-11-14 | Premium | «Prevents the author of a merge request from approving it.» |
| L04 | https://docs.gitlab.com/user/project/merge_requests/approvals/rules/ | 2026-11-14 | Premium | «Merge request approval rules Tier : Premium, Ultimate» |
| L05 | https://docs.gitlab.com/user/project/codeowners/ | 2026-11-14 | Premium | «Code Owners Tier : Premium, Ultimate» |
| L06 | https://docs.gitlab.com/user/project/merge_requests/methods/ | 2026-11-14 | Free | «Squashing is never performed, and the user cannot change the behavior.» |
| L07 | https://docs.gitlab.com/ci/pipelines/merge_trains/ | 2026-11-14 | Premium | «Use merge trains to put merge requests in a queue.» |
| L08 | https://docs.gitlab.com/user/project/merge_requests/auto_merge/ | 2026-11-14 | Free | «This setting also prevents merge requests from merging if there is no pipeline» |
| L09 | https://docs.gitlab.com/user/project/merge_requests/status_checks/ | 2026-11-14 | Ultimate | «External status checks Tier : Ultimate» |
| L10 | https://docs.gitlab.com/user/project/protected_tags/ | 2026-11-14 | Free | «Prevent accidental update or deletion once created.» |
| L11 | https://docs.gitlab.com/user/project/settings/project_access_tokens/ | 2026-11-14 | Premium | «On GitLab.com, project access tokens require a Premium or Ultimate subscription.» |
| L12 | https://docs.gitlab.com/user/profile/service_accounts/ | 2026-11-14 | Free | «Service accounts on Free tier: Introduced in GitLab 18.10» |
| L13 | https://docs.gitlab.com/api/commits/ | 2026-11-14 | Free | «Must be 255 characters or fewer.» |
| L14 | https://docs.gitlab.com/user/project/repository/signed_commits/ | 2026-11-14 | Free | «the commit is marked Verified in the GitLab UI» |
| L15 | https://docs.gitlab.com/api/merge_request_approvals/ | 2026-11-14 | Free | «Merge request approvals API Tier : Free, Premium, Ultimate» |
| L16 | https://docs.gitlab.com/api/users/ | 2026-11-14 | Free | «"bot" : false» |
| L17 | https://docs.gitlab.com/ci/jobs/ci_job_token/ | 2026-11-14 | Free | «Can access the GET /projects/:id/merge_requests» |
| L18 | https://docs.gitlab.com/api/pipelines/ | 2026-11-14 | Free | «Return pipelines for the specified commit SHA.» |
| L19 | https://docs.gitlab.com/api/merge_requests/ | 2026-11-14 | Free | «response includes the commits behind the target branch» |
| L20 | https://docs.gitlab.com/ci/pipelines/merge_request_pipelines/ | 2026-11-14 | Free | «Is created and runs in the fork (source) project» |
| L21 | https://docs.gitlab.com/ci/variables/ | 2026-11-14 | Free | «only available in pipelines that run on protected branches or protected tags» |
| L22 | https://docs.gitlab.com/ci/runners/configure_runners/ | 2026-11-14 | Free | «You can use tags to control the jobs a runner can run.» |
| L23 | https://docs.gitlab.com/user/permissions/ | 2026-11-14 | Free | «Create commit status» |
| L24 | https://docs.gitlab.com/ci/ci_cd_for_external_repos/external_commit_statuses/ | 2026-11-14 | Free | «External commit statuses allow external CI/CD systems» |
| L25 | https://docs.gitlab.com/ci/pipelines/merged_results_pipelines/ | 2026-11-14 | Premium | «Merged results pipelines Tier : Premium, Ultimate» |
| L26 | https://about.gitlab.com/pricing/ | 2026-11-14 | precios | «5 users per top-level group» |
| L27 | https://docs.gitlab.com/user/project/repository/push_rules/ | 2026-11-14 | Premium | «Push rules Tier : Premium, Ultimate» |
| L28 | https://docs.gitlab.com/ci/pipelines/settings/ | 2026-11-14 | Free | «Is in a different project, include the group and project name.» |
<!-- fuentes:fin -->

Notas de lectura (paráfrasis, no citas):

- D02 también da, en la misma página, «Restrict deletions», «Block force
  pushes», la aprobación del último push, el método de merge permitido, el
  origen de un check («select an app as the expected source») y «Require
  branches to be up to date before merging».
- D03: la cola de merge exige el evento `merge_group` en los workflows.
- L01: «Allowed to force push» está apagado por defecto; borrar una rama
  protegida lo puede hacer un Maintainer u Owner desde la interfaz o la API,
  no con `git`.
- L02: en Free las aprobaciones son opcionales; «Required approvals» es
  Premium.
- L03: «Prevent approvals by users who add commits» y «Remove all approvals
  when commits are added to the source branch» son Premium (misma página).
- L06: el método de merge (y «Merge commit with semi-linear history») se
  configura **por proyecto**, en Settings > Merge requests.
- L11: en self-managed, los tokens de proyecto están disponibles con
  cualquier licencia; crean un usuario bot.
- L12: las cuentas de servicio en Free, disponibles en general desde 18.11.
- L15: la cabecera de la API es Free; «Retrieve approval details» y las
  reglas de aprobación del MR son Premium.
- L17: el job token solo llega a una lista cerrada de endpoints; en
  merge requests, solo los dos GET.
- L23: «Create commit status» lo tiene el rol Developer y superiores.
- L24: el estado externo se añade a un pipeline del commit; que bloquee el
  merge con «Pipelines must succeed» no lo dice la página (no verificado).
