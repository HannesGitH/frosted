// generated file, do not edit

part of 'in2.model.dart';

// ------------------------------------------------------------

extension UiInsurancePreviewSimpleDataModelCopyWith on UiInsurancePreviewSimpleDataModel {
  UiInsurancePreviewSimpleDataModel Function({
    String title,
    String cta,
    String? image,
  }) get copyWith => ({
        Object? title = Never,
        Object? cta = Never,
        Object? image = Never,
      }) =>
          UiInsurancePreviewSimpleDataModel(
            title: title._or(this.title),
            cta: cta._or(this.cta),
            image: image._or(this.image),
          );
}
// ------------------------------------------------------------

extension UiInsurancePreviewDataModelCopyWith on UiInsurancePreviewDataModel {
  UiInsurancePreviewDataModel Function({
    String? avatar,
    String tagType,
  }) get copyWith => ({
        Object? avatar = Never,
        Object? tagType = Never,
      }) =>
          UiInsurancePreviewDataModel(
            avatar: avatar._or(this.avatar),
            tagType: tagType._or(this.tagType),
          );
}
// ------------------------------------------------------------

extension on Object? {
  // T? _as<T>() => this is T ? this as T : null;
  T _or<T>(T other) => this is T ? this as T : other;
}
