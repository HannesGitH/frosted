// generated file, do not edit

part of 'in2.model.dart';

// ------------------------------------------------------------
mixin UiInsurancePreviewSimpleDataModelCopyWith {
  String get title;
  String get cta;
  String? get image;

  UiInsurancePreviewSimpleDataModel copyWith({
    String? title,
    String? cta,
    NullableValue<String?>? image,
  }) => UiInsurancePreviewSimpleDataModel(
    title: title  ??  this.title,
    cta: cta  ??  this.cta,
    image: image  |  this.image,
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
